use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

pub(crate) struct LoudnessInput {
    pub(crate) source: Entity,
    pub(crate) listener: Entity,
}

pub(crate) fn loudness_at(
    In(LoudnessInput { source, listener }): In<LoudnessInput>,
) -> Result<f32> {
    Ok(0.0)
}
