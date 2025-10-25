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
    context::StreamRestartEvent,
    pool::{Sampler, SamplerPool},
    prelude::*,
    sample_effects,
};
use bevy_steam_audio::{
    nodes::{SteamAudioNode, SteamAudioPool},
    simulation::AudionimbusSimulator,
    sources::AudionimbusSource,
};
use firewheel::{
    channel_config::ChannelConfig,
    diff::{Diff, EventQueue as _, Patch, RealtimeClone},
    event::{NodeEventType, ProcEvents},
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
        .add_observer(despawn_pool_late)
        .add_observer(reestablish_channel);
    app.register_required_components::<AiPool, AiAudible>();
    app.register_node::<InputBufferNode>();
}

#[derive(Component)]
pub(crate) struct InputBuffer {
    pub(crate) inputs: Vec<f32>,
    pub(crate) loudness: f32,
    is_dropped: Arc<AtomicBool>,
    cons: Mutex<Cons>,
}

impl InputBuffer {
    pub fn update_loudness(&mut self) {
        self.loudness = rms(&self.inputs);
    }
}

#[derive(PoolLabel, Reflect, PartialEq, Eq, Debug, Hash, Clone)]
#[reflect(Component)]
pub(crate) struct AiPool;

#[derive(Component, Diff, Patch, Clone, RealtimeClone, Default, Reflect)]
#[reflect(Component)]
struct InputBufferNode;

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
    mut buffers: Query<&mut InputBuffer>,
    time: Res<Time>,
    mut scratch: Local<Option<Vec<f32>>>,
) {
    let scratch = scratch.get_or_insert_with(|| vec![0.0; param::MAX_FRAME_SIZE as usize]);
    for mut buffer in buffers.iter_mut() {
        let is_dropped = buffer.is_dropped.load(Ordering::Relaxed);

        if is_dropped {
            let silence = ((time.delta_secs() * param::SAMPLING_RATE as f32).floor() as usize)
                .min(param::MAX_FRAME_SIZE as usize);

            buffer.inputs.drain(..silence);
            buffer.inputs.extend(iter::repeat_n(0.0, silence));
            buffer.update_loudness();
            continue;
        }

        let status = {
            let Ok(mut cons) = buffer.cons.try_lock() else {
                error!("Node cons not unlocked");
                continue;
            };
            cons.read_interleaved(scratch)
        };
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
            continue;
        }
        buffer.inputs.drain(..incoming);
        buffer.inputs.extend(&scratch[..incoming]);
        buffer.update_loudness();

        if buffer.inputs.capacity() > MAX_FRAME_SIZE as usize {
            error!("Input buffer capacity exceeded {MAX_FRAME_SIZE}");
        }
    }
}

fn establish_channel(
    nodes: Query<(Entity, &SampleEffects), Added<Sampler>>,
    mut input_buffer_nodes: Query<&mut AudioEvents, With<InputBufferNode>>,
    main_simulator: Res<AudionimbusSimulator>,
    mut commands: Commands,
) {
    for (entity, effects) in nodes.iter() {
        let Ok(mut events) = input_buffer_nodes.get_effect_mut(effects) else {
            continue;
        };
        let (prod, cons) = fixed_resample::resampling_channel(
            1.try_into().unwrap(),
            main_simulator.sampling_rate.get(),
            param::SAMPLING_RATE,
            resampling_channel_config(),
        );
        let is_dropped = Arc::new(AtomicBool::new(false));
        let event = InputBufferInitEvent {
            prod: Some(prod),
            is_dropped: Some(is_dropped.clone()),
        };
        events.push(NodeEventType::custom(event));

        commands.entity(entity).insert(InputBuffer {
            inputs: vec![0.0; param::MAX_FRAME_SIZE as usize],
            loudness: 0.0,
            is_dropped,
            cons: Mutex::new(cons),
        });
    }
}

fn reestablish_channel(
    restart: On<StreamRestartEvent>,
    mut nodes: Query<(&SampleEffects, &mut InputBuffer)>,
    mut input_buffer_nodes: Query<&mut AudioEvents, With<InputBufferNode>>,
    main_simulator: Res<AudionimbusSimulator>,
) {
    if restart.event().previous_rate == restart.event().current_rate {
        return;
    }
    for (effects, mut input_buffer) in nodes.iter_mut() {
        let Ok(mut events) = input_buffer_nodes.get_effect_mut(effects) else {
            continue;
        };
        let (prod, cons) = fixed_resample::resampling_channel(
            1.try_into().unwrap(),
            main_simulator.sampling_rate.get(),
            param::SAMPLING_RATE,
            resampling_channel_config(),
        );
        let event = InputBufferInitEvent {
            prod: Some(prod),
            is_dropped: None,
        };
        events.push(NodeEventType::custom(event));

        input_buffer.cons = Mutex::new(cons);
    }
}

struct InputBufferInitEvent {
    prod: Option<Prod>,
    is_dropped: Option<Arc<AtomicBool>>,
}

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
        InputBufferProcessor {
            prod: None,
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
    prod: Option<Prod>,
    is_prod_dropped: Option<Arc<AtomicBool>>,
    mono_buffer: Vec<f32>,
}

impl AudioNodeProcessor for InputBufferProcessor {
    fn process(
        &mut self,
        _proc_info: &ProcInfo,
        ProcBuffers { inputs, .. }: ProcBuffers,
        events: &mut ProcEvents,
        _extra: &mut ProcExtra,
    ) -> firewheel::node::ProcessStatus {
        for mut event in events.drain() {
            if let Some(out_stream_event) = event.downcast_mut::<InputBufferInitEvent>() {
                // Swap the values so that the old producer gets dropped on
                // the main thread.
                core::mem::swap(&mut self.prod, &mut out_stream_event.prod);
                if let Some(new_prod_dropped) = out_stream_event.is_dropped.take() {
                    let old_prod_dropped = self.is_prod_dropped.replace(new_prod_dropped);
                    if let Some(old_prod_dropped) = old_prod_dropped {
                        old_prod_dropped.store(true, Ordering::Relaxed);
                    }
                }
            }
        }
        let Some(prod) = self.prod.as_mut() else {
            return ProcessStatus::Bypass;
        };

        // Don't early return on empty input: that is a valid thing to buffer.

        // downsample from stereo to mono
        self.mono_buffer.clear();
        for (l, r) in inputs[0].iter().zip(inputs[1]) {
            self.mono_buffer.push((l + r) / 2.0);
        }

        let status = prod.push_interleaved(&self.mono_buffer);
        match status {
            fixed_resample::PushStatus::Ok => {}
            fixed_resample::PushStatus::OutputNotReady => {}
            fixed_resample::PushStatus::OverflowOccurred { num_frames_pushed } => {
                warn!("Overflow while pushing data: {num_frames_pushed}")
            }
            fixed_resample::PushStatus::UnderflowCorrected {
                num_zero_frames_pushed,
            } => warn!("Underflow while pushing data: {num_zero_frames_pushed}"),
        }
        ProcessStatus::Bypass
    }

    fn new_stream(
        &mut self,
        stream_info: &firewheel::StreamInfo,
        _context: &mut firewheel::node::ProcStreamCtx,
    ) {
        if stream_info.sample_rate == stream_info.prev_sample_rate {
            return;
        }

        // We could drop self.prod here and wait until the ECS sends a new one, but that would drop frames.
        // Better imo to resample some inputs at the wrong rate than to drop them.
    }
}
