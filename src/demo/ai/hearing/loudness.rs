use bevy::prelude::*;
use bevy_seedling::prelude::{EffectsQuery, SampleEffects};

use crate::demo::ai::hearing::node::InputBuffer;

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

pub(crate) struct LoudnessInput {
    pub(crate) listener: Entity,
    pub(crate) source: Entity,
}

pub(crate) fn loudness_to_listener(
    In(LoudnessInput { listener, source }): In<LoudnessInput>,
    mut commands: Commands,
    input_buffer: Query<&InputBuffer>,
) -> Result<f32> {
    let buffer = input_buffer.get(source)?;
    let loudness = buffer.loudness;
    info!("Loudness: {}", loudness);
    Ok(loudness)
}
