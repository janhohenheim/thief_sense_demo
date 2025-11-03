use bevy::prelude::*;
use bevy_steam_audio::STEAM_AUDIO_CONTEXT;

use crate::demo::{
    ai::hearing::debug::{EnableAudioPathVisualization, EnableAudioWriter},
    npc::Npc,
};

pub(crate) mod accumulator;
mod bookkeeping;
mod debug;
pub(crate) mod listen;
mod loudness;
pub(crate) mod node;
mod simulate;
pub(crate) mod source_of;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AiSimulator>();
    app.add_plugins((
        bookkeeping::plugin,
        simulate::plugin,
        loudness::plugin,
        listen::plugin,
        node::plugin,
        debug::plugin,
        accumulator::plugin,
        source_of::plugin,
    ));
    app.insert_resource(EnableAudioPathVisualization(false))
        .insert_resource(EnableAudioWriter(cfg!(feature = "dev")));
    app.register_required_components::<Npc, LoudnessAcuity>();
}

/// This is to [`AiSource`] what [`RigidBody`] is to [`Collider`].
///
/// [`RigidBody`]: avian3d::prelude::RigidBody
/// [`Collider`]: avian3d::prelude::Collider
#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
pub(crate) struct AiSourceBody;

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub(crate) struct LoudnessAcuity(pub(crate) f32);

impl Default for LoudnessAcuity {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub(crate) struct AiLoudnessControl {
    pub(crate) low_loudness: u8,
    pub(crate) medium_loudness: u8,
    pub(crate) high_loudness: u8,
}

impl Default for AiLoudnessControl {
    fn default() -> Self {
        Self {
            low_loudness: 50,
            medium_loudness: 70,
            high_loudness: 90,
        }
    }
}

mod param {
    use crate::demo::ai::sense::{SENSE_INTERVAL_FAR, SENSE_INTERVAL_NEAR};

    /// Nyquist frequency is 4k, that's two octaves above 1k, which the human ear is most sensitive to.
    pub(super) const SAMPLING_RATE: u32 = 8000;
    pub(super) const FRAME_SIZE_NEAR: u32 = ((SAMPLING_RATE as f32) * SENSE_INTERVAL_NEAR) as u32;
    pub(super) const FRAME_SIZE_FAR: u32 = ((SAMPLING_RATE as f32) * SENSE_INTERVAL_FAR) as u32;
    pub(super) const MAX_FRAME_SIZE: u32 = FRAME_SIZE_FAR;
    pub(super) const MIN_FRAME_SIZE: u32 = FRAME_SIZE_NEAR;

    pub(super) const ORDER: u32 = 1;
    pub(super) const CHANNELS: u32 = (ORDER + 1) * (ORDER + 1);
    pub(super) const FLAGS: audionimbus::SimulationFlags =
        audionimbus::SimulationFlags::from_bits_truncate(
            audionimbus::SimulationFlags::DIRECT.bits()
                | audionimbus::SimulationFlags::PATHING.bits(),
        );
    pub(super) const AUDIO_SETTINGS: audionimbus::AudioSettings = audionimbus::AudioSettings {
        sampling_rate: SAMPLING_RATE,
        frame_size: MIN_FRAME_SIZE,
    };
}

type Simulator =
    audionimbus::Simulator<audionimbus::Direct, audionimbus::Reflections, audionimbus::Pathing>;

#[derive(Debug, Clone, Resource, Deref, DerefMut)]
struct AiSimulator(Simulator);

impl FromWorld for AiSimulator {
    fn from_world(_world: &mut World) -> Self {
        Self(
            audionimbus::Simulator::builder(
                audionimbus::SceneParams::Default,
                param::SAMPLING_RATE,
                param::MIN_FRAME_SIZE,
            )
            .with_direct(audionimbus::DirectSimulationSettings {
                // We use raycasts, not volumetric
                max_num_occlusion_samples: 0,
            })
            // TODO: pretend we use reflections until https://github.com/MaxenceMaire/audionimbus/pull/31
            .with_reflections(audionimbus::ReflectionsSimulationSettings::Convolution {
                max_order: param::ORDER, // <- The important bit :)
                max_num_rays: 0,
                num_diffuse_samples: 0,
                max_duration: 0.0,
                max_num_sources: 0,
                num_threads: 0,
            })
            .with_pathing(audionimbus::PathingSimulationSettings {
                num_visibility_samples: 8,
            })
            .try_build(&STEAM_AUDIO_CONTEXT)
            .unwrap(),
        )
    }
}

#[derive(Component, Clone, Deref, DerefMut)]
#[require(Transform, GlobalTransform, AiLoudnessControl)]
struct AiSource(audionimbus::Source);

#[inline]
fn rms(samples: &[f32]) -> f32 {
    let sum = samples.iter().copied().map(|x| x * x).sum::<f32>();
    (sum / samples.len() as f32).sqrt()
}
