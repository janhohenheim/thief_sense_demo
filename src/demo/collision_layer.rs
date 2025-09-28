use avian3d::prelude::PhysicsLayer;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

#[derive(Debug, PhysicsLayer, Default)]
pub(crate) enum CollisionLayer {
    #[default]
    Default,
    AiVisible,
    LightSource,
    Transparent,
}
