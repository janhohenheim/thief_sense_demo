use std::{collections::VecDeque, fs::File, io::BufWriter, sync::Mutex};

use bevy::{ecs::relationship::Relationship, prelude::*};
use bevy_seedling::{
    SeedlingSystems, pool::SamplerPool, prelude::*, sample::SamplePlayer, sample_effects,
};
use bevy_steam_audio::nodes::{FixedProcessBlock, SteamAudioNode};
use firewheel::{
    channel_config::ChannelConfig,
    diff::{Diff, EventQueue as _, Patch, RealtimeClone},
    event::{NodeEventType, ProcEvents},
    node::{
        AudioNode, AudioNodeProcessor, EmptyConfig, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
    },
};
use hound::{SampleFormat, WavSpec, WavWriter};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Producer, Split},
};
use rubato::{FastFixedOut, PolynomialDegree, Resampler};

use crate::demo::ai::hearing::{AiAudible, FRAME_SIZE_FAR, SAMPLING_RATE};
type Prod = <HeapRb<f32> as Split>::Prod;
type Cons = <HeapRb<f32> as Split>::Cons;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreStartup, init_pool)
        .add_systems(PreUpdate, update_input_buffer)
        .add_systems(Last, establish_channel.in_set(SeedlingSystems::Queue));
    app.register_required_components::<AiPool, AiAudible>();
    app.add_systems(Update, write);
    app.register_node::<InputBufferNode>();
    app.add_observer(init_writer).add_observer(finish_writer);
}

fn init_writer(add: On<Add, InputBuffer>, mut commands: Commands) -> Result {
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLING_RATE,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };

    let writer = WavWriter::create("output.wav", spec)?;
    commands.entity(add.entity).insert(Writer(Some(writer)));
    info!("Initialized writer");
    Ok(())
}

fn write(mut writer: Query<(&InputBuffer, &mut Writer), Changed<InputBuffer>>) -> Result {
    for (buff, mut writer) in writer.iter_mut() {
        for sample in &buff.inputs {
            writer.0.as_mut().unwrap().write_sample(*sample)?;
        }
    }
    info!("Wrote samples");
    Ok(())
}

fn finish_writer(remove: On<Remove, Writer>, mut writer: Query<&mut Writer>) -> Result {
    let mut writer = writer.get_mut(remove.entity)?;
    writer.0.take().unwrap().finalize()?;
    info!("Finished writer");
    panic!();
    Ok(())
}

#[derive(Component)]
struct Writer(Option<WavWriter<BufWriter<File>>>);

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

fn update_input_buffer(mut buffers: Query<&mut InputBuffer>) {
    for mut buffer in buffers.iter_mut() {
        let mut scratch = [0.0; FRAME_SIZE_FAR as usize];
        let incoming = buffer.cons.lock().unwrap().pop_slice(&mut scratch);
        if incoming == 0 {
            // be kind to change detection
            continue;
        }
        buffer.inputs.drain(..incoming);
        buffer.inputs.extend(scratch);
        buffer.update_loudness();
    }
}

fn establish_channel(
    mut nodes: Query<(Entity, &SampleEffects), Added<SamplePlayer>>,
    mut input_buffers: Query<&mut AudioEvents, With<InputBufferNode>>,
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
                num_outputs: ChannelCount::STEREO,
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
        let chunk_size = FRAME_SIZE_FAR as usize;
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
            fixed_block: FixedProcessBlock::new(44104 as usize, 0, 2, 0),
            resample_in: [vec![0.0; 44104 as usize]; 1],
            resample_out: [vec![0.0; FRAME_SIZE_FAR as usize]; 1],
        }
    }
}

struct InputBufferProcessor {
    prod: Option<Prod>,
    resampler: FastFixedOut<f32>,
    fixed_block: FixedProcessBlock,
    resample_in: [Vec<f32>; 1],
    resample_out: [Vec<f32>; 1],
}

impl AudioNodeProcessor for InputBufferProcessor {
    fn process(
        &mut self,
        proc_info: &ProcInfo,
        proc_buffers: ProcBuffers,
        events: &mut ProcEvents,
        _extra: &mut ProcExtra,
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
        let fixed_block = &mut self.fixed_block;
        let temp_proc = ProcBuffers {
            inputs: proc_buffers.inputs,
            outputs: proc_buffers.outputs,
        };
        fixed_block.process(temp_proc, proc_info, |inputs, _outputs| {
            for (i, sample) in self.resample_in[0].iter_mut().enumerate() {
                *sample = (inputs[0][i] + inputs[1][i]) / 2.0;
            }
            let delay = self.resampler.output_delay();
            let new_length =
                (FRAME_SIZE_FAR * SAMPLING_RATE / proc_info.sample_rate.get()) as usize;
            self.resampler
                .process_into_buffer(&mut self.resample_in, &mut self.resample_out, None)
                .unwrap();
            prod.push_slice(&self.resample_out[0][delay..(delay + new_length)]);
        });
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
