use std::{collections::VecDeque, sync::Mutex};

use bevy::{ecs::relationship::Relationship, prelude::*};
use bevy_seedling::{
    SeedlingSystems, pool::SamplerPool, prelude::*, sample::SamplePlayer, sample_effects,
};
use bevy_steam_audio::nodes::SteamAudioNode;
use firewheel::{
    channel_config::ChannelConfig,
    diff::{Diff, EventQueue as _, Patch},
    event::{NodeEventType, ProcEvents},
    node::{
        AudioNode, AudioNodeProcessor, EmptyConfig, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
    },
};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Producer, Split},
};
use rubato::{FastFixedOut, PolynomialDegree, VecResampler};

use crate::demo::ai::hearing::FRAME_SIZE_FAR;
type Prod = <HeapRb<f32> as Split>::Prod;
type Cons = <HeapRb<f32> as Split>::Cons;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreStartup, init_pool)
        .add_systems(PreUpdate, update_input_buffer)
        .add_systems(Last, establish_channel.in_set(SeedlingSystems::Queue));
}

#[derive(Component)]
pub(crate) struct InputBuffer {
    inputs: VecDeque<f32>,
    loudness: f32,
    cons: Mutex<Cons>,
}

impl InputBuffer {
    pub fn update_loudness(&mut self) {
        let sum_of_squares = self.inputs.iter().map(|&x| x * x).sum::<f32>();
        self.loudness = (sum_of_squares / self.inputs.len() as f32).sqrt()
    }
}

#[derive(PoolLabel, Reflect, PartialEq, Eq, Debug, Hash, Clone)]
#[reflect(Component)]
pub(crate) struct AiPool;

#[derive(Debug, Default, Clone, Component, Diff, Patch, Reflect)]
struct InputBufferNode;

fn init_pool(mut commands: Commands) {
    commands.spawn((
        Name::new("Music audio sampler pool"),
        SamplerPool(AiPool),
        sample_effects![InputBufferNode, SteamAudioNode::default(),],
    ));
}

fn update_input_buffer(mut buffers: Query<&mut InputBuffer>) {
    for mut buffer in buffers.iter_mut() {
        let mut scratch = [0.0; FRAME_SIZE_FAR as usize];
        let incoming = buffer.cons.lock().unwrap().pop_slice(&mut scratch);
        buffer.inputs.drain(..incoming);
        buffer.inputs.extend(scratch);
        buffer.update_loudness();
    }
}

fn establish_channel(
    mut nodes: Query<(Entity, &SampleEffects), Added<SamplePlayer>>,
    mut input_buffers: Query<&mut AudioEvents, With<InputBuffer>>,
    mut commands: Commands,
) {
    for (entity, effects) in nodes.iter_mut() {
        let Ok(mut events) = input_buffers.get_effect_mut(effects) else {
            continue;
        };
        let (prod, cons) = HeapRb::new(FRAME_SIZE_FAR as usize).split();
        let event = InputBufferInitEvent(Some(prod));
        events.push(NodeEventType::custom(event));
        commands.entity(entity).insert(InputBuffer {
            inputs: VecDeque::with_capacity(FRAME_SIZE_FAR as usize),
            loudness: 0.0,
            cons: Mutex::new(cons),
        });
    }
}

struct InputBufferInitEvent(Option<Prod>);

impl AudioNode for InputBufferNode {
    type Configuration = EmptyConfig;

    fn info(&self, _configuration: &Self::Configuration) -> firewheel::node::AudioNodeInfo {
        firewheel::node::AudioNodeInfo::new()
            .debug_name("input buffer")
            .channel_config(ChannelConfig {
                num_inputs: ChannelCount::STEREO,
                num_outputs: ChannelCount::ZERO,
            })
    }

    fn construct_processor(
        &self,
        _configuration: &Self::Configuration,
        cx: firewheel::node::ConstructProcessorContext,
    ) -> impl firewheel::node::AudioNodeProcessor {
        let resample_ratio = FRAME_SIZE_FAR as f64 / cx.stream_info.sample_rate.get() as f64;
        let max_resample_ratio_relative = 1.0;
        let interpolation_type = PolynomialDegree::Linear;
        let chunk_size = cx.stream_info.max_block_frames.get() as usize;
        let nbr_channels = 1;
        InputBufferProcessor {
            prod: None,
            resampler: FastFixedOut::new(
                resample_ratio,
                max_resample_ratio_relative,
                interpolation_type,
                chunk_size,
                nbr_channels,
            )
            .unwrap(),
        }
    }
}

struct InputBufferProcessor {
    prod: Option<Prod>,
    resampler: FastFixedOut<f32>,
}

impl AudioNodeProcessor for InputBufferProcessor {
    fn process(
        &mut self,
        proc_info: &ProcInfo,
        ProcBuffers { inputs, .. }: ProcBuffers,
        events: &mut ProcEvents,
        extra: &mut ProcExtra,
    ) -> firewheel::node::ProcessStatus {
        for mut event in events.drain() {
            if let Some(out_stream_event) = event.downcast_mut::<InputBufferInitEvent>() {
                // Swap the values so that the old producer gets dropped on
                // the main thread.
                core::mem::swap(&mut self.prod, &mut out_stream_event.0);
            }
        }

        if proc_info.in_silence_mask.all_channels_silent(2) {
            return ProcessStatus::Bypass;
        }
        let Some(prod) = self.prod.as_mut() else {
            return ProcessStatus::Bypass;
        };
        let scratch = extra
            .scratch_buffers
            .first_with_frames_mut(proc_info.frames);
        for (i, sample) in scratch.iter_mut().enumerate() {
            *sample = (inputs[0][i] + inputs[1][i]) / 2.0;
        }
        self.resampler
            .process_into_buffer(todo!(), todo!(), todo!());
        prod.push_slice(&scratch[..inputs[0].len()]);
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
        let resample_ratio = FRAME_SIZE_FAR as f64 / stream_info.sample_rate.get() as f64;
        let max_resample_ratio_relative = 1.0;
        let interpolation_type = PolynomialDegree::Linear;
        let chunk_size = stream_info.max_block_frames.get() as usize;
        let nbr_channels = 1;
        self.resampler = FastFixedOut::new(
            resample_ratio,
            max_resample_ratio_relative,
            interpolation_type,
            chunk_size,
            nbr_channels,
        )
        .unwrap();
    }
}
