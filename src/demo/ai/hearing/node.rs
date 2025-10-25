use std::{
    iter,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use bevy_seedling::{
    SeedlingSystems,
    pool::{Sampler, SamplerPool},
    prelude::*,
    sample_effects,
};
use bevy_steam_audio::{
    nodes::{SteamAudioNode, SteamAudioPool},
    sources::AudionimbusSource,
};
use firewheel::{
    channel_config::ChannelConfig,
    diff::{Diff, EventQueue as _, Patch},
    event::ProcEvents,
    node::{
        AudioNode, AudioNodeProcessor, EmptyConfig, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
    },
};
use fixed_resample::{ResampleQuality, ResamplingChannelConfig};

use crate::{
    demo::ai::{
        hearing::{
            AiAudible,
            param::{self, MAX_FRAME_SIZE},
            rms,
        },
        sense::SENSE_INTERVAL_FAR,
    },
    despawn::Despawn,
};
type Prod = fixed_resample::ResamplingProd<f32, 1>;
type Cons = fixed_resample::ResamplingCons<f32>;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreStartup, init_pool)
        .add_systems(FixedPreUpdate, update_input_buffer)
        .add_systems(Last, establish_channel.in_set(SeedlingSystems::Queue));
    app.add_observer(setup_sample_player)
        .add_observer(despawn_pool_late);
    app.register_required_components::<AiPool, AiAudible>();
    app.register_node::<InputBufferNode>();
}

#[derive(Component)]
pub(crate) struct InputBuffer {
    pub(crate) inputs: Vec<f32>,
    pub(crate) loudness: f32,
    is_dropped: Arc<AtomicBool>,
}

impl InputBuffer {
    pub fn update_loudness(&mut self) {
        self.loudness = rms(&self.inputs);
    }
}

#[derive(PoolLabel, Reflect, PartialEq, Eq, Debug, Hash, Clone)]
#[reflect(Component)]
pub(crate) struct AiPool;

#[derive(Component, Diff, Patch, Clone, Default, Reflect)]
#[reflect(Component)]
struct InputBufferNode {
    #[reflect(ignore)]
    #[diff(skip)]
    cons: Arc<Mutex<Option<Cons>>>,
}

fn init_pool(mut commands: Commands) {
    commands.spawn((
        Name::new("AI sound pool"),
        SamplerPool(AiPool),
        sample_effects![InputBufferNode::default(), SteamAudioNode::default(),],
    ));
}

fn setup_sample_player(
    add: On<Add, AiPool>,
    mut commands: Commands,
    playback_settings: Query<&PlaybackSettings, Allow<Disabled>>,
) {
    let mut settings = if let Ok(settings) = playback_settings.get(add.entity) {
        settings.clone()
    } else {
        PlaybackSettings::default()
    };
    settings.on_complete = OnComplete::Remove;
    commands.entity(add.entity).insert(settings);
}

fn despawn_pool_late(remove: On<Remove, AiPool>, mut commands: Commands) {
    commands
        .entity(remove.entity)
        .try_remove::<SteamAudioPool>()
        .try_remove::<AudionimbusSource>()
        .try_insert(Despawn::after(SENSE_INTERVAL_FAR * 10.0));
}

fn update_input_buffer(
    mut buffers: Query<(&mut InputBuffer, &SampleEffects, Has<Despawn>)>,
    node: Query<&InputBufferNode>,
    time: Res<Time>,
    mut scratch: Local<Option<Vec<f32>>>,
) {
    let scratch = scratch.get_or_insert_with(|| vec![0.0; param::MAX_FRAME_SIZE as usize]);
    for (mut buffer, effects, despawn) in buffers.iter_mut() {
        let Ok(node) = node.get_effect(effects) else {
            error!("Input buffer has no node");
            continue;
        };
        let is_dropped = buffer.is_dropped.load(Ordering::Relaxed);
        if is_dropped {
            assert!(despawn);
        }
        if despawn && is_dropped {
            let silence = ((time.delta_secs() * param::SAMPLING_RATE as f32).floor() as usize)
                .min(param::MAX_FRAME_SIZE as usize);

            buffer.inputs.drain(..silence);
            buffer.inputs.extend(iter::repeat_n(0.0, silence));
            buffer.update_loudness();
        } else {
            loop {
                let Ok(mut cons) = node.cons.try_lock() else {
                    error!("Node cons not unlocked");
                    break;
                };
                let Some(cons) = cons.as_mut() else {
                    error!("Node processor not ready");
                    break;
                };
                let status = cons.read_interleaved(scratch);
                let incoming = match status {
                    fixed_resample::ReadStatus::Ok => param::MAX_FRAME_SIZE as usize,
                    fixed_resample::ReadStatus::InputNotReady => 0,
                    fixed_resample::ReadStatus::UnderflowOccurred { num_frames_read } => {
                        // This is entirely expected: the producer and consumer run at different rates.
                        num_frames_read
                    }
                    fixed_resample::ReadStatus::OverflowCorrected {
                        num_frames_discarded,
                    } => {
                        warn!("Overflow in input buffer: {num_frames_discarded} frames discarded");
                        param::MAX_FRAME_SIZE as usize
                    }
                };
                if incoming == 0 {
                    break;
                }
                buffer.inputs.drain(..incoming);
                buffer.inputs.extend(&scratch[..incoming]);
                buffer.update_loudness();
            }
        }
        if buffer.inputs.capacity() > MAX_FRAME_SIZE as usize {
            error!("Input buffer capacity exceeded {MAX_FRAME_SIZE}");
        }
    }
}

fn establish_channel(
    mut nodes: Query<(Entity, &SampleEffects), Added<Sampler>>,
    mut input_buffers: Query<&mut AudioEvents, With<InputBufferNode>>,
    mut commands: Commands,
) {
    for (entity, effects) in nodes.iter_mut() {
        let Ok(events) = input_buffers.get_effect_mut(effects) else {
            continue;
        };
        // Todo: attach a new node to the processor
        // - new (prod, cons)
        // - tell the old it's dropped

        let is_dropped = Arc::new(AtomicBool::new(false));
        commands.entity(entity).insert(InputBuffer {
            inputs: vec![0.0; param::MAX_FRAME_SIZE as usize],
            loudness: 0.0,
            is_dropped,
        });
    }
}

struct InputBufferInitEvent {
    prod: Prod,
    is_dropped: Arc<AtomicBool>,
}

const FIXED_BLOCK_SIZE: usize = 1024;

impl AudioNode for InputBufferNode {
    type Configuration = EmptyConfig;

    fn info(&self, _configuration: &Self::Configuration) -> firewheel::node::AudioNodeInfo {
        firewheel::node::AudioNodeInfo::new()
            .debug_name("input buffer")
            .channel_config(ChannelConfig {
                num_inputs: ChannelCount::STEREO,
                num_outputs: ChannelCount::STEREO,
            })
    }

    fn construct_processor(
        &self,
        _configuration: &Self::Configuration,
        cx: firewheel::node::ConstructProcessorContext,
    ) -> impl firewheel::node::AudioNodeProcessor {
        let (prod, cons) = fixed_resample::resampling_channel(
            1.try_into().unwrap(),
            cx.stream_info.sample_rate.get(),
            param::SAMPLING_RATE,
            resampling_channel_config(),
        );
        self.cons.try_lock().unwrap().replace(cons);

        InputBufferProcessor {
            prod,
            is_prod_dropped: None,
            mono_buffer: Vec::with_capacity(cx.stream_info.max_block_frames.get() as usize),
        }
    }
}

fn resampling_channel_config() -> ResamplingChannelConfig {
    ResamplingChannelConfig {
        quality: ResampleQuality::Low,
        underflow_autocorrect_percent_threshold: None,
        ..default()
    }
}

struct InputBufferProcessor {
    prod: Prod,
    is_prod_dropped: Option<Arc<AtomicBool>>,
    mono_buffer: Vec<f32>,
}

impl AudioNodeProcessor for InputBufferProcessor {
    fn process(
        &mut self,
        proc_info: &ProcInfo,
        ProcBuffers { inputs, .. }: ProcBuffers,
        events: &mut ProcEvents,
        _extra: &mut ProcExtra,
    ) -> firewheel::node::ProcessStatus {
        for mut event in events.drain() {
            if let Some(out_stream_event) = event.downcast_mut::<InputBufferInitEvent>() {
                // Swap the values so that the old producer gets dropped on
                // the main thread.
                core::mem::swap(&mut self.prod, &mut out_stream_event.prod);
                let old_prod_dropped = self
                    .is_prod_dropped
                    .replace(out_stream_event.is_dropped.clone());
                if let Some(old_prod_dropped) = old_prod_dropped {
                    old_prod_dropped.store(true, Ordering::Relaxed);
                }
            }
        }

        // Don't early return on empty input: that is a valid thing to buffer.

        // downsample from stereo to mono
        self.mono_buffer.clear();
        for (l, r) in inputs[0].iter().zip(inputs[1]) {
            self.mono_buffer.push((l + r) / 2.0);
        }

        let status = self.prod.push_interleaved(&self.mono_buffer);
        match status {
            fixed_resample::PushStatus::Ok => {}
            fixed_resample::PushStatus::OutputNotReady => {}
            fixed_resample::PushStatus::OverflowOccurred { num_frames_pushed } => {
                error!("Underflow while pushing data: {num_frames_pushed}")
            }
            fixed_resample::PushStatus::UnderflowCorrected {
                num_zero_frames_pushed,
            } => error!("Underflow while pushing data: {num_zero_frames_pushed}"),
        }
        ProcessStatus::Bypass
    }

    fn new_stream(
        &mut self,
        stream_info: &firewheel::StreamInfo,
        _context: &mut firewheel::node::ProcStreamCtx,
    ) {
        if stream_info.sample_rate == stream_info.prev_sample_rate {}
        // TODO: (prod, cons) pair is now invalid
    }
}
