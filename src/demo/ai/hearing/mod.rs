use bevy::prelude::*;
use bevy_steam_audio::STEAM_AUDIO_CONTEXT;

use crate::demo::ai::sense::{SENSE_INTERVAL_FAR, SENSE_INTERVAL_NEAR};

mod bookkeeping;
pub(crate) mod listen;
mod loudness;
mod node;
mod simulate;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AiSimulators>();
    app.add_plugins((
        bookkeeping::plugin,
        simulate::plugin,
        loudness::plugin,
        listen::plugin,
        node::plugin,
    ));
}

/// Nyquist frequency is 4k, that's two octaves above 1k, which the human ear is most sensitive to.
const SAMPLING_RATE: u32 = 8000;
const FRAME_SIZE_NEAR: u32 = ((SAMPLING_RATE as f32) * SENSE_INTERVAL_NEAR) as u32;
const FRAME_SIZE_FAR: u32 = ((SAMPLING_RATE as f32) * SENSE_INTERVAL_FAR) as u32;

mod param {
    pub(super) const ORDER: u32 = 1;
    pub(super) const FLAGS: audionimbus::SimulationFlags =
        audionimbus::SimulationFlags::from_bits_truncate(
            audionimbus::SimulationFlags::DIRECT.bits()
                | audionimbus::SimulationFlags::PATHING.bits(),
        );
}

type Simulator = audionimbus::Simulator<audionimbus::Direct, (), audionimbus::Pathing>;

#[derive(Debug, Clone, Resource)]
struct AiSimulators {
    near: Simulator,
    far: Simulator,
}

impl AiSimulators {
    fn iter_mut(&mut self) -> impl IntoIterator<Item = &mut Simulator> {
        [&mut self.near, &mut self.far]
    }
}

impl FromWorld for AiSimulators {
    fn from_world(_world: &mut World) -> Self {
        let simulator = |frame_size: u32| {
            audionimbus::Simulator::builder(
                audionimbus::SceneParams::Default,
                SAMPLING_RATE,
                frame_size,
            )
            .with_direct(audionimbus::DirectSimulationSettings {
                // We use raycasts, not volumetric
                max_num_occlusion_samples: 0,
            })
            .with_pathing(audionimbus::PathingSimulationSettings {
                num_visibility_samples: 8,
            })
            .try_build(&STEAM_AUDIO_CONTEXT)
            .unwrap()
        };
        Self {
            near: simulator(FRAME_SIZE_NEAR),
            far: simulator(FRAME_SIZE_FAR),
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct AiAudible;

#[derive(Component, Clone)]
#[require(Transform, GlobalTransform)]
struct AiSources {
    near: audionimbus::Source,
    far: audionimbus::Source,
}
