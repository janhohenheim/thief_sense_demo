use bevy::{ecs::world::DeferredWorld, prelude::*};

use crate::rand_timer::{RandTimer, RandTimerApp};

pub(super) fn plugin(app: &mut App) {
    app.add_rand_timer::<VisibilityTimer>();
}

#[derive(Component, Deref, DerefMut)]
struct VisibilityTimer(RandTimer);

impl Default for VisibilityTimer {
    fn default() -> Self {
        Self(RandTimer::from_millis(200))
    }
}

#[derive(Component, Debug)]
pub(crate) struct AiVisibility {
    lighting: f32,
    movement: f32,
    exposure: f32,
}

pub(crate) fn get_or_update_visibility(In(entity): In<Entity>) -> Result<AiVisibility> {
    Ok(AiVisibility {
        lighting: 0.0,
        movement: 0.0,
        exposure: 0.0,
    })
}
