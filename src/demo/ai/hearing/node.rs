use bevy::prelude::*;
use bevy_seedling::prelude::ChannelCount;
use firewheel::{
    channel_config::ChannelConfig,
    collector::ArcGc,
    diff::{Diff, Patch},
    event::ProcEvents,
    node::{AudioNode, AudioNodeProcessor, EmptyConfig, ProcBuffers, ProcExtra, ProcInfo},
};

use crate::demo::ai::hearing::{FRAME_SIZE_FAR, FRAME_SIZE_NEAR};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

/// A node that analyzes the loudness of an incoming signal.
#[derive(Debug, Default, Clone, Component, Diff, Patch, Reflect)]
struct InputBufferNode;

#[derive(Debug)]
struct InnerState {
    input_near: [f32; FRAME_SIZE_NEAR as usize],
    input_far: [f32; FRAME_SIZE_FAR as usize],
    loudness_near: f32,
    loudness_far: f32,
}

#[derive(Debug, Clone)]
pub struct InputBufferState(ArcGc<InnerState>);

impl AudioNode for InputBufferNode {
    type Configuration = EmptyConfig;

    fn info(&self, _configuration: &Self::Configuration) -> firewheel::node::AudioNodeInfo {
        firewheel::node::AudioNodeInfo::new()
            .debug_name("input buffer")
            .channel_config(ChannelConfig {
                num_inputs: ChannelCount::STEREO,
                num_outputs: ChannelCount::ZERO,
            })
            .custom_state(InputBufferState(ArcGc::new(InnerState {
                input_near: [0.0; FRAME_SIZE_NEAR as usize],
                input_far: [0.0; FRAME_SIZE_FAR as usize],
                loudness_near: 0.0,
                loudness_far: 0.0,
            })))
    }

    fn construct_processor(
        &self,
        _configuration: &Self::Configuration,
        cx: firewheel::node::ConstructProcessorContext,
    ) -> impl firewheel::node::AudioNodeProcessor {
        InputBufferProcessor {
            state: cx.custom_state().cloned().unwrap(),
        }
    }
}

struct InputBufferProcessor {
    state: InputBufferState,
}

impl AudioNodeProcessor for InputBufferProcessor {
    fn process(
        &mut self,
        proc_info: &ProcInfo,
        buffers: ProcBuffers,
        events: &mut ProcEvents,
        _: &mut ProcExtra,
    ) -> firewheel::node::ProcessStatus {
        firewheel::node::ProcessStatus::Bypass
    }
}
