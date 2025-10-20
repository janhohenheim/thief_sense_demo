use bevy::{ecs::system::RunSystemOnce, prelude::*};
use bevy_steam_audio::wrapper::AudionimbusCoordinateSystem;

use crate::demo::ai::hearing::{
    AiSource,
    simulate::{AiSimulationInputs, update_simulation_for_listener},
};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

pub(crate) struct LoudnessInput {
    pub(crate) listener: Entity,
    pub(crate) source: Entity,
}

pub(crate) fn loudness_to_listener(
    In(LoudnessInput { listener, source }): In<LoudnessInput>,
    world: &mut World,
) -> Result<f32> {
    todo!()
}
