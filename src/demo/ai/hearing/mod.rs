use bevy::prelude::*;
use bevy_steam_audio::STEAM_AUDIO_CONTEXT;

use crate::demo::ai::hearing::debug::{EnableAudioPathVisualization, EnableAudioWriter};

mod bookkeeping;
mod debug;
pub(crate) mod listen;
mod loudness;
pub(crate) mod node;
mod simulate;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AiSimulators>();
    app.add_plugins((
        bookkeeping::plugin,
        simulate::plugin,
        loudness::plugin,
        listen::plugin,
        node::plugin,
        debug::plugin,
    ));
    app.insert_resource(EnableAudioPathVisualization(false))
        .insert_resource(EnableAudioWriter(true));
}

mod param {
    use crate::demo::ai::sense::{SENSE_INTERVAL_FAR, SENSE_INTERVAL_NEAR};

    /// Nyquist frequency is 4k, that's two octaves above 1k, which the human ear is most sensitive to.
    pub(super) const SAMPLING_RATE: u32 = 8000;
    pub(super) const FRAME_SIZE_NEAR: u32 = ((SAMPLING_RATE as f32) * SENSE_INTERVAL_NEAR) as u32;
    pub(super) const FRAME_SIZE_FAR: u32 = ((SAMPLING_RATE as f32) * SENSE_INTERVAL_FAR) as u32;
    pub(super) const MAX_FRAME_SIZE: u32 = FRAME_SIZE_FAR;

    pub(super) const ORDER: u32 = 1;
    pub(super) const CHANNELS: u32 = (ORDER + 1) * (ORDER + 1);
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
                param::SAMPLING_RATE,
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
            near: simulator(param::FRAME_SIZE_NEAR),
            far: simulator(param::FRAME_SIZE_FAR),
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct AiAudible;

#[derive(Component, Clone)]
#[require(Transform, GlobalTransform)]
struct AiSources {
    near: audionimbus::Source,
    far: audionimbus::Source,
}

#[inline]
fn rms(samples: &[f32]) -> f32 {
    let sum = samples.iter().copied().map(|x| x * x).sum::<f32>();
    (sum / samples.len() as f32).sqrt()
}
