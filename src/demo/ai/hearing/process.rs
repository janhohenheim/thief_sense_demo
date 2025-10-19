use bevy::prelude::*;

use crate::demo::ai::hearing::AiSource;

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

pub(crate) fn loudness_at(In((source, listener)): In<(AiSource, Transform)>) -> Result<f32> {
    Ok(0.0)
}
