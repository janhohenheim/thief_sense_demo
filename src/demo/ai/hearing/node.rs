use bevy::prelude::*;
use bevy_seedling::prelude::ChannelCount;
use firewheel::{
    atomic_float::AtomicF64,
    channel_config::ChannelConfig,
    collector::ArcGc,
    diff::{Diff, Patch},
    node::{AudioNode, AudioNodeProcessor, EmptyConfig},
};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

/// A node that analyzes the loudness of an incoming signal.
#[derive(Debug, Default, Clone, Component, Diff, Patch, Reflect)]
struct InputBufferNode;

#[derive(Debug)]
struct InnerState {
    input: [f32; FRAME_SIZE as usize],
}

#[derive(Debug, Clone)]
pub struct InputBufferState(ArcGc<InnerState>);

impl AudioNode for InputBufferNode {
    type Configuration = EmptyConfig;

    fn info(&self, _configuration: &Self::Configuration) -> firewheel::node::AudioNodeInfo {
        firewheel::node::AudioNodeInfo::new()
            .debug_name("input buffer")
            .channel_config(ChannelConfig {
                num_inputs: ChannelCount::MONO,
                num_outputs: ChannelCount::ZERO,
            })
            .custom_state(InputBufferState(ArcGc::new(InnerState {
                input: [0.0; FRAME_SIZE as usize],
            })))
    }

    fn construct_processor(
        &self,
        configuration: &Self::Configuration,
        cx: firewheel::node::ConstructProcessorContext,
    ) -> impl firewheel::node::AudioNodeProcessor {
        InputBufferProcessor {
            analyzer: construct_analyzer(
                cx.stream_info.sample_rate.get(),
                configuration.channel_map.as_deref(),
            ),
            ignore_silence: configuration.ignore_silence,
            channel_map: configuration.channel_map.clone(),
            state: cx.custom_state().cloned().unwrap(),
        }
    }
}

struct InputBufferProcessor {
    analyzer: EbuR128,
    ignore_silence: bool,
    channel_map: Option<Vec<Channel>>,
    state: LoudnessState,
}

impl AudioNodeProcessor for InputBufferProcessor {
    fn process(
        &mut self,
        proc_info: &ProcInfo,
        buffers: ProcBuffers,
        events: &mut ProcEvents,
        _: &mut ProcExtra,
    ) -> firewheel::node::ProcessStatus {
        for LoudnessNodePatch::Reset(_) in events.drain_patches::<LoudnessNode>() {
            self.analyzer.reset();
        }

        if self.ignore_silence
            && proc_info
                .in_silence_mask
                .all_channels_silent(buffers.inputs.len())
        {
            return firewheel::node::ProcessStatus::Bypass;
        }

        self.analyzer
            .add_frames_planar_f32(buffers.inputs)
            .expect("input channels should match configuration");

        let state = &self.state.0;
        state
            .integrated
            .store(self.analyzer.loudness_global().unwrap(), Ordering::Relaxed);
        state.momentary.store(
            self.analyzer.loudness_momentary().unwrap(),
            Ordering::Relaxed,
        );
        state.short_term.store(
            self.analyzer.loudness_shortterm().unwrap(),
            Ordering::Relaxed,
        );
        state
            .loudness_range
            .store(self.analyzer.loudness_range().unwrap(), Ordering::Relaxed);

        for i in 0..buffers.inputs.len() {
            state.sample_peak[i].store(
                self.analyzer.sample_peak(i as u32).unwrap(),
                Ordering::Relaxed,
            );

            state.true_peak[i].store(
                self.analyzer.true_peak(i as u32).unwrap(),
                Ordering::Relaxed,
            );
        }

        firewheel::node::ProcessStatus::Bypass
    }

    fn new_stream(&mut self, stream_info: &firewheel::StreamInfo) {
        if stream_info.sample_rate != stream_info.prev_sample_rate {
            // unfortunately, we have to re-construct here
            self.analyzer =
                construct_analyzer(stream_info.sample_rate.get(), self.channel_map.as_deref());
        }
    }
}
