use std::{f32::consts::TAU, sync::LazyLock};

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{demo::collision_layer::CollisionLayer, rand_timer::RandTimer};

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

static SPHERE: LazyLock<Collider> = LazyLock::new(|| Collider::sphere(1.0));

pub(crate) fn get_or_update_visibility(
    In(entity): In<Entity>,
    mut timers: Query<(&mut AiVisibility, &mut VisibilityTimer)>,
    object: Query<(&GlobalTransform, &LinearVelocity)>,
    transforms: Query<&GlobalTransform>,
    spatial: SpatialQuery,
) -> Result<AiVisibility> {
    let (mut visibility, mut timer) = timers.get_mut(entity)?;
    if !timer.is_finished() {
        return Ok(*visibility);
    }
    let (transform, velocity) = object.get(entity)?;
    let translation = transform.translation();

    // lighting
    let lights = spatial.shape_intersections(
        &SPHERE,
        translation,
        Quat::IDENTITY,
        &SpatialQueryFilter::from_mask([CollisionLayer::LightSource]),
    );
    let mut lighting = 0.0;
    let filter =
        SpatialQueryFilter::from_mask([CollisionLayer::Opaque]).with_excluded_entities([entity]);
    for light in lights {
        let Ok(light_transform) = transforms.get(light) else {
            continue;
        };
        let Ok((dir, len)) = Dir3::new_and_length(light_transform.translation() - translation)
        else {
            lighting += 1.0;
            continue;
        };
        let hit = spatial.cast_ray(translation, dir, len, true, &filter);
        if hit.is_some() {
            continue;
        }
        let light_translation = light_transform.translation();
        let distance = translation.distance(light_translation);
        lighting += 1.0 / distance;
    }

    // movement
    let movement = velocity.length();

    // exposure
    let closest_wall = calc_closest_wall(translation, spatial, entity);
    let exposure = closest_wall;

    *visibility = AiVisibility {
        lighting,
        movement,
        exposure,
    };
    // Since this system is not called every frame, but only for entities that are currently looked at by AI,
    // we only reset the timer when necessary.
    timer.reset();
    Ok(*visibility)
}

fn calc_closest_wall(translation: Vec3, spatial: SpatialQuery, entity: Entity) -> f32 {
    const TRIES: u8 = 8;
    // This distance is equivalent to "infinitely far away"
    const MAX_WALL_DISTANCE: f32 = 5.0;
    let mut closest_wall = MAX_WALL_DISTANCE;
    let filter =
        SpatialQueryFilter::from_mask([CollisionLayer::Static]).with_excluded_entities([entity]);
    for i in 0..TRIES {
        let dir = Quat::from_rotation_y(i as f32 * TAU / TRIES as f32) * Dir3::NEG_Z;
        let hit = spatial.cast_ray(translation, dir, MAX_WALL_DISTANCE, true, &filter);
        if let Some(hit) = hit {
            closest_wall = closest_wall.min(hit.distance);
        }
    }
    closest_wall
}

fn tick_visibility_timer(mut timers: Query<&mut VisibilityTimer>, time: Res<Time>) {
    for mut timer in timers.iter_mut() {
        timer.tick(&time);
    }
}
