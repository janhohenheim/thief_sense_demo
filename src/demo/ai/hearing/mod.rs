use bevy::prelude::*;
use bevy_steam_audio::STEAM_AUDIO_CONTEXT;
use std::{
    array,
    sync::{Arc, RwLock, atomic::AtomicBool},
};

mod bookkeeping;
mod process;
mod run;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AiSimulators>();
    app.add_plugins((bookkeeping::plugin, run::plugin, process::plugin));
}

/// Nyquist frequency is 4k, that's two octaves above 1k, which the human ear is most sensitive to.
const SAMPLING_RATE: u32 = 8000;
/// in seconds
const HEARING_INTERVAL: f32 = 0.2;
const FRAME_SIZE: u32 = ((SAMPLING_RATE as f32) * HEARING_INTERVAL) as u32;

mod param {
    pub(super) const ORDER: u32 = 1;
    pub(super) const FLAGS: audionimbus::SimulationFlags =
        audionimbus::SimulationFlags::from_bits_truncate(
            audionimbus::SimulationFlags::DIRECT.bits()
                | audionimbus::SimulationFlags::PATHING.bits(),
        );
}

const BUCKET_SIZE: usize = 10;

#[derive(Debug, Clone, Resource, Deref, DerefMut)]
struct AiSimulators(
    [audionimbus::Simulator<audionimbus::Direct, (), audionimbus::Pathing>; BUCKET_SIZE],
);

impl FromWorld for AiSimulators {
    fn from_world(_world: &mut World) -> Self {
        let simulator = |_| {
            audionimbus::Simulator::builder(
                audionimbus::SceneParams::Default,
                SAMPLING_RATE,
                FRAME_SIZE,
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
        Self(array::from_fn(simulator))
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct AiAudible;

#[derive(Component, Deref, Clone, DerefMut)]
#[require(Transform, GlobalTransform)]
struct AiSource(pub(crate) audionimbus::Source);
