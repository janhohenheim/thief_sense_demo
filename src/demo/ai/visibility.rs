use std::f32::consts::TAU;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    collision_layer::CollisionLayer,
    cpu_lighting::estimate_tone_mapped_lighting,
    demo::player::{PLAYER_RUN_SPEED, PLAYER_WALK_SPEED},
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

#[derive(Component, Debug, Copy, Clone)]
pub(crate) struct AiVisibilityControl {
    pub(crate) low_visibility: u8,
    pub(crate) medium_visibility: u8,
    pub(crate) high_visibility: u8,

    pub(crate) low_speed: f32,
    pub(crate) high_speed: f32,

    pub(crate) low_speed_mod: u8,
    pub(crate) medium_speed_mod: u8,
    pub(crate) high_speed_mod: u8,

    pub(crate) wall_dist: f32,
    pub(crate) wall_mod: u8,
}

impl Default for AiVisibilityControl {
    fn default() -> Self {
        Self {
            low_visibility: 13,
            medium_visibility: 19,
            high_visibility: 44,
            low_speed: PLAYER_WALK_SPEED - 0.01,
            high_speed: PLAYER_RUN_SPEED,
            low_speed_mod: 0,
            medium_speed_mod: 5,
            high_speed_mod: 10,
            wall_dist: 0.5,
            wall_mod: 1,
        }
    }
}

#[derive(Component, Debug, Copy, Clone, Default)]
#[require(VisibilityTimer, AiVisibilityControl)]
pub(crate) struct AiVisibility {
    pub(crate) lighting: u8,
    pub(crate) movement: u8,
    pub(crate) exposure: u8,
}

impl AiVisibility {
    pub(crate) fn level(self) -> u8 {
        (self.lighting + self.movement + self.exposure).min(100)
    }
}

pub(crate) fn get_or_update_visibility(
    In(entity): In<Entity>,
    world: &mut World,
) -> Result<AiVisibility> {
    if !world
        .entity(entity)
        .get::<VisibilityTimer>()
        .ok_or("No Timer on Entity")?
        .is_finished()
    {
        return Ok(*world
            .entity(entity)
            .get::<AiVisibility>()
            .ok_or("No Visibility on Entity")?);
    }

    let raw_lighting = world.run_system_cached_with(estimate_tone_mapped_lighting, entity)?;
    let lighting = world.run_system_cached_with(calculate_light_rating, (entity, raw_lighting))?;

    let movement = world.run_system_cached_with(calculate_movement_rating, entity)?;

    let exposure = world.run_system_cached_with(calculate_exposure_rating, entity)?;

    // Since this system is not called every frame, but only for entities that are currently looked at by AI,
    // we only reset the timer when necessary.
    let mut entity_mut = world.entity_mut(entity);
    entity_mut
        .get_mut::<VisibilityTimer>()
        .ok_or("No Timer on Entity")?
        .reset();

    let mut visibility = entity_mut
        .get_mut::<AiVisibility>()
        .ok_or("No Visibility on Entity")?;
    *visibility = AiVisibility {
        lighting,
        movement,
        exposure,
    };
    Ok(*visibility)
}

fn calculate_exposure_rating(
    In(entity): In<Entity>,
    spatial: SpatialQuery,
    object: Query<(&AiVisibilityControl, &GlobalTransform)>,
) -> Result<u8> {
    let (control, transform) = object.get(entity)?;
    let translation = transform.translation();

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

    let exposure = if closest_wall < control.wall_dist {
        control.wall_mod
    } else {
        0
    };
    Ok(exposure)
}

fn calculate_movement_rating(
    In(entity): In<Entity>,
    object: Query<(&AiVisibilityControl, &LinearVelocity)>,
) -> Result<u8> {
    let (control, velocity) = object.get(entity)?;
    let movement = match velocity.length_squared() {
        v if v < control.low_speed => control.low_speed_mod,
        v if v > control.high_speed => control.high_speed_mod,
        _ => control.medium_speed_mod,
    };
    Ok(movement)
}

fn calculate_light_rating(
    In((entity, raw_lighting)): In<(Entity, f32)>,
    object: Query<&AiVisibilityControl>,
) -> Result<u8> {
    let control = object.get(entity)?;
    let raw_lighting = (raw_lighting * 100.0).clamp(1.0, 100.0) as u8;
    const LOW_LIGHT_NORM: u8 = 25;
    const MEDIUM_LIGHT_NORM: u8 = 50;
    const HIGH_LIGHT_NORM: u8 = 75;
    let (pre_norm_base, pre_norm_range, norm_base, norm_range) = match raw_lighting {
        l if l < control.low_visibility => (0, control.low_visibility, 0, LOW_LIGHT_NORM),
        l if l < control.medium_visibility => (
            control.low_visibility,
            control.medium_visibility - control.low_visibility,
            LOW_LIGHT_NORM,
            MEDIUM_LIGHT_NORM - LOW_LIGHT_NORM,
        ),
        l if l < control.high_visibility => (
            control.medium_visibility,
            control.high_visibility - control.medium_visibility,
            MEDIUM_LIGHT_NORM,
            HIGH_LIGHT_NORM - MEDIUM_LIGHT_NORM,
        ),
        _ => (
            control.high_visibility,
            100 - control.high_visibility,
            HIGH_LIGHT_NORM,
            100 - HIGH_LIGHT_NORM,
        ),
    };
    let result = norm_base
        + ((raw_lighting - pre_norm_base) as f32 / pre_norm_range as f32) as u8
        + norm_range;
    Ok(result)
}

fn tick_visibility_timer(mut timers: Query<&mut VisibilityTimer>, time: Res<Time>) {
    for mut timer in timers.iter_mut() {
        timer.tick(&time);
    }
}
