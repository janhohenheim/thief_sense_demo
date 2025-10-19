use bevy::prelude::*;
use bevy_steam_audio::STEAM_AUDIO_CONTEXT;
use std::sync::{Arc, RwLock, atomic::AtomicBool};

mod bookkeeping;
mod process;
mod run;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AiSimulator>();
    app.add_plugins((bookkeeping::plugin, run::plugin, process::plugin));
}

/// Nyquist frequency is 4k, that's two octaves above 1k, which the human ear is most sensitive to.
const SAMPLING_RATE: u32 = 8000;
/// in seconds
const HEARING_INTERVAL: f32 = 0.2;
const FRAME_SIZE: u32 = ((SAMPLING_RATE as f32) * HEARING_INTERVAL) as u32;

mod param {
    pub(super) const REFLECT_RAYS: u32 = 1024;
    pub(super) const REFLECT_BOUNCES: u32 = 4;
    pub(super) const REFLECT_DURATION: f32 = 1.0;
    pub(super) const ORDER: u32 = 1;
    pub(super) const FLAGS: audionimbus::SimulationFlags =
        audionimbus::SimulationFlags::from_bits_truncate(
            audionimbus::SimulationFlags::DIRECT.bits() | EXPENSIVE_FLAGS.bits(),
        );
    pub(super) const EXPENSIVE_FLAGS: audionimbus::SimulationFlags =
        audionimbus::SimulationFlags::from_bits_truncate(
            audionimbus::SimulationFlags::REFLECTIONS.bits(),
        );
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct AiSimulator(
    Arc<RwLock<audionimbus::Simulator<audionimbus::Direct, audionimbus::Reflections>>>,
);

impl FromWorld for AiSimulator {
    fn from_world(_world: &mut World) -> Self {
        let simulator = audionimbus::Simulator::builder(
            audionimbus::SceneParams::Default,
            SAMPLING_RATE,
            FRAME_SIZE,
        )
        .with_direct(audionimbus::DirectSimulationSettings {
            // We use raycasts, not volumetric
            max_num_occlusion_samples: 0,
        })
        .with_reflections(audionimbus::ReflectionsSimulationSettings::Convolution {
            max_num_rays: param::REFLECT_RAYS,
            num_diffuse_samples: 1024,
            max_duration: param::REFLECT_DURATION,
            max_order: param::ORDER,
            max_num_sources: 200,
            num_threads: 1,
        })
        .try_build(&STEAM_AUDIO_CONTEXT)
        .unwrap();
        Self(Arc::new(RwLock::new(simulator)))
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct AiAudible;

#[derive(Component, Deref, Clone, DerefMut)]
#[require(Transform, GlobalTransform)]
struct AiSource(pub(crate) audionimbus::Source);

#[derive(Resource)]
struct AiAsyncSimulationSynchronization {
    sender: crossbeam_channel::Sender<()>,
    complete: Arc<AtomicBool>,
}
