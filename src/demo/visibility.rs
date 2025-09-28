use std::f32::consts::TAU;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_trenchbroom::geometry::Brushes;

use crate::{
    demo::{collision_layer::CollisionLayer, player::PLAYER_RADIUS},
    rand_timer::RandTimer,
};

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

pub(crate) fn get_or_update_visibility(
    In(entity): In<Entity>,
    mut timers: Query<(&mut AiVisibility, &mut VisibilityTimer)>,
    object: Query<(&GlobalTransform, &Collider, &LinearVelocity)>,
    spatial: SpatialQuery,
) -> Result<AiVisibility> {
    let (mut visibility, mut timer) = timers.get_mut(entity)?;
    if !timer.is_finished() {
        return Ok(*visibility);
    }
    let (transform, collider, velocity) = object.get(entity)?;
    let translation = transform.translation();
    const TRIES: u8 = 8;
    // This distance is equivalent to "infinitely far away"
    const MAX_WALL_DISTANCE: f32 = 5.0;
    let mut closest_wall = MAX_WALL_DISTANCE;
    for i in 0..TRIES {
        let dir = Quat::from_rotation_y(i as f32 * TAU / TRIES as f32) * Dir3::NEG_Z;
        let hit = spatial.cast_ray(
            translation,
            dir,
            MAX_WALL_DISTANCE,
            true,
            &SpatialQueryFilter::from_mask([CollisionLayer::Brush]),
        );
        if let Some(hit) = hit {
            closest_wall = closest_wall.min(hit.distance);
        }
    }
    *visibility = AiVisibility {
        // TODO: do some raycasts. Can probably first do an AABB or sphere check to gather the potential light sources. Also remember to filter out `CollisionLayer::Transparent` from raycasts.
        lighting: 0.0,
        movement: velocity.length(),
        exposure: closest_wall,
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
