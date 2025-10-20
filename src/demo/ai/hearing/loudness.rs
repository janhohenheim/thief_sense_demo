use bevy::prelude::*;

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
