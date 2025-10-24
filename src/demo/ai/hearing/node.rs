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
    diff::{Diff, EventQueue as _, Patch, RealtimeClone},
    event::{NodeEventType, ProcEvents},
    node::{
        AudioNode, AudioNodeProcessor, EmptyConfig, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
    },
};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Producer, Split},
};
use rubato::{FastFixedOut, PolynomialDegree, Resampler};

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
type Prod = <HeapRb<f32> as Split>::Prod;
type Cons = <HeapRb<f32> as Split>::Cons;

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
    cons: Mutex<Cons>,
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

#[derive(Diff, Patch, Debug, PartialEq, Clone, RealtimeClone, Component, Reflect)]
#[reflect(Component)]
struct InputBufferNode;

fn init_pool(mut commands: Commands) {
    commands.spawn((
        Name::new("AI sound pool"),
        SamplerPool(AiPool),
        sample_effects![InputBufferNode, SteamAudioNode::default(),],
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

fn update_input_buffer(mut buffers: Query<(&mut InputBuffer, Has<Despawn>)>, time: Res<Time>) {
    let mut scratch = [0.0; param::MAX_FRAME_SIZE as usize];
    for (mut buffer, despawn) in buffers.iter_mut() {
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
                let incoming = buffer.cons.lock().unwrap().pop_slice(&mut scratch);
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
        let Ok(mut events) = input_buffers.get_effect_mut(effects) else {
            continue;
        };
        // This should be BLOCK_SIZE * n for some wiggle room for when the audio thread is pushing faster than we can process.
        // Turns out `MAX_FRAME_SIZE` fits the bill nicely!
        let ringbuf_capacity = param::MAX_FRAME_SIZE as usize;
        let (prod, cons) = HeapRb::new(ringbuf_capacity).split();
        let is_dropped = Arc::new(AtomicBool::new(false));
        let event = InputBufferInitEvent {
            prod: Some(prod),
            is_dropped: is_dropped.clone(),
        };
        events.push(NodeEventType::custom(event));
        commands.entity(entity).insert(InputBuffer {
            inputs: vec![0.0; param::MAX_FRAME_SIZE as usize],
            loudness: 0.0,
            cons: Mutex::new(cons),
            is_dropped,
        });
    }
}

struct InputBufferInitEvent {
    prod: Option<Prod>,
    is_dropped: Arc<AtomicBool>,
}

const FIXED_BLOCK_SIZE: usize = 1024;

impl AudioNode for InputBufferNode {
    type Configuration = EmptyConfig;

    fn info(&self, _configuration: &Self::Configuration) -> firewheel::node::AudioNodeInfo {
        firewheel::node::AudioNodeInfo::new()
            .debug_name("input buffer")
            .channel_config(ChannelConfig {
                num_inputs: ChannelCount::MONO,
                num_outputs: ChannelCount::STEREO,
            })
    }

    fn construct_processor(
        &self,
        _configuration: &Self::Configuration,
        cx: firewheel::node::ConstructProcessorContext,
    ) -> impl firewheel::node::AudioNodeProcessor {
        let resample_ratio = param::SAMPLING_RATE as f64 / cx.stream_info.sample_rate.get() as f64;
        let max_resample_ratio_relative = 1.0;
        let interpolation_type = PolynomialDegree::Linear;
        let chunk_size = (resample_ratio * FIXED_BLOCK_SIZE as f64).floor() as usize;
        let nbr_channels = 1;
        let resampler = FastFixedOut::new(
            resample_ratio,
            max_resample_ratio_relative,
            interpolation_type,
            chunk_size,
            nbr_channels,
        )
        .unwrap();
        InputBufferProcessor {
            prod: None,
            is_prod_dropped: None,
            resampler,
            fixed_input: Vec::with_capacity(FIXED_BLOCK_SIZE),
            fixed_out: vec![0.0; chunk_size],
        }
    }
}

struct InputBufferProcessor {
    prod: Option<Prod>,
    is_prod_dropped: Option<Arc<AtomicBool>>,
    resampler: FastFixedOut<f32>,
    fixed_input: Vec<f32>,
    fixed_out: Vec<f32>,
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
        if self.fixed_input.capacity() > FIXED_BLOCK_SIZE {
            error_once!("Allocated fixed_input in audio thread");
        }
        if self.fixed_out.len() > FIXED_BLOCK_SIZE {
            error_once!("Allocated fixed_out in audio thread");
        }

        // Don't early return on empty input: that is a valid thing to buffer.

        if self.fixed_input.len() + proc_info.frames > FIXED_BLOCK_SIZE {
            let diff = proc_info.frames + self.fixed_input.len() - FIXED_BLOCK_SIZE;
            self.fixed_input.drain(..diff);
        }
        for sample in inputs[0] {
            // downsample from stereo to mono
            self.fixed_input.push(*sample * 0.5);
        }
        if self.fixed_input.len() < self.fixed_input.capacity() {
            return ProcessStatus::Bypass;
        }
        let Some(prod) = self.prod.as_mut() else {
            return ProcessStatus::Bypass;
        };

        let rms_before = rms(&self.fixed_input);
        let (read, written) = self
            .resampler
            .process_into_buffer(&[&self.fixed_input], &mut [&mut self.fixed_out], None)
            .unwrap();
        self.fixed_input.drain(..read);
        let out = &mut self.fixed_out[..written];

        let rms_after = rms(out);
        if rms_after > 1e-5 {
            let ratio = rms_before / rms_after;
            for sample in out.iter_mut() {
                *sample *= ratio;
            }
        }
        prod.push_slice(out);
        ProcessStatus::Bypass
    }

    fn new_stream(
        &mut self,
        stream_info: &firewheel::StreamInfo,
        _context: &mut firewheel::node::ProcStreamCtx,
    ) {
        if stream_info.sample_rate == stream_info.prev_sample_rate {
            return;
        };
        let resample_ratio = param::SAMPLING_RATE as f64 / stream_info.sample_rate.get() as f64;
        let max_resample_ratio_relative = 1.0;
        let interpolation_type = PolynomialDegree::Linear;
        let chunk_size = (resample_ratio * FIXED_BLOCK_SIZE as f64).floor() as usize;
        let nbr_channels = 1;
        self.resampler = FastFixedOut::new(
            resample_ratio,
            max_resample_ratio_relative,
            interpolation_type,
            chunk_size,
            nbr_channels,
        )
        .unwrap();
        self.fixed_out = vec![0.0; chunk_size];
    }
}
