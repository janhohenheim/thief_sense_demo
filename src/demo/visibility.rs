use avian3d::prelude::LinearVelocity;
use bevy::{ecs::world::DeferredWorld, prelude::*};

use crate::rand_timer::{RandTimer, RandTimerApp};

pub(super) fn plugin(app: &mut App) {
    // NOT calling `add_rand_timer` because we want to manually reset it
    app.add_systems(PreUpdate, tick_visibility_timer);
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct VisibilityTimer(RandTimer);

impl Default for VisibilityTimer {
    fn default() -> Self {
        let mut timer = RandTimer::from_millis(200);
        // The visibility starts off uncalculated,
        // so start the timer finished in order to always "re"calculate the visibility when it's first requested.
        timer.finish();
        Self(timer)
    }
}

#[derive(Component, Debug, Copy, Clone, Default)]
#[require(VisibilityTimer)]
pub(crate) struct AiVisibility {
    pub(crate) lighting: f32,
    pub(crate) movement: f32,
    pub(crate) exposure: f32,
}

// Todo:
// - Add AiVisibility to NPCs
//
pub(crate) fn get_or_update_visibility(
    In(entity): In<Entity>,
    mut timers: Query<(&mut AiVisibility, &mut VisibilityTimer)>,
    object: Query<&LinearVelocity>,
) -> Result<AiVisibility> {
    let (mut visibility, mut timer) = timers.get_mut(entity)?;
    if !timer.is_finished() {
        return Ok(*visibility);
    }
    let velocity = object.get(entity)?;
    *visibility = AiVisibility {
        lighting: 0.0,
        movement: velocity.length(),
        exposure: 0.0,
    };
    // Since this system is not called every frame, but only for entities that are currently looked at by AI,
    // we only reset the timer when necessary.
    timer.reset();
    Ok(*visibility)
}

fn tick_visibility_timer(mut timers: Query<&mut VisibilityTimer>, time: Res<Time>) {
    for mut timer in timers.iter_mut() {
        timer.tick(&time);
    }
}
