use std::{f32::consts::TAU, sync::LazyLock};

use avian3d::prelude::*;
use bevy::{math::FloatPow, prelude::*};

use crate::{
    cpu_lighting::{estimate_directional_light, estimate_point_light, estimate_spot_light},
    demo::collision_layer::CollisionLayer,
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
pub(crate) struct AiVisibilityControl {
    pub(crate) low_visibility: u32,
    pub(crate) medium_visibility: u32,
    pub(crate) high_visibility: u32,

    pub(crate) low_speed: f32,
    pub(crate) high_speed: f32,

    pub(crate) low_speed_mod: u32,
    pub(crate) medium_speed_mod: u32,
    pub(crate) high_speed_mod: u32,

    pub(crate) wall_dist: f32,
    pub(crate) wall_mod: u32,
}

#[derive(Component, Debug, Copy, Clone, Default)]
#[require(VisibilityTimer, AiVisibilityControl)]
#[expect(dead_code, reason = "Needs to be implemented!")]
pub(crate) struct AiVisibility {
    pub(crate) lighting: u32,
    pub(crate) movement: u32,
    pub(crate) exposure: u32,
}

static SPHERE: LazyLock<Collider> = LazyLock::new(|| Collider::sphere(1.0));

pub(crate) fn get_or_update_visibility(
    In(entity): In<Entity>,
    mut timers: Query<(
        &mut AiVisibility,
        &AiVisibilityControl,
        &mut VisibilityTimer,
    )>,
    object: Query<(&GlobalTransform, &LinearVelocity)>,
    lights: Query<(
        &GlobalTransform,
        NameOrEntity,
        AnyOf<(&PointLight, &SpotLight)>,
    )>,
    directional_lights: Query<(&GlobalTransform, &DirectionalLight)>,
    spatial: SpatialQuery,
) -> Result<AiVisibility> {
    let (mut visibility, control, mut timer) = timers.get_mut(entity)?;
    if !timer.is_finished() {
        return Ok(*visibility);
    }
    let (transform, velocity) = object.get(entity)?;
    let translation = transform.translation();

    // lighting
    let lighting = calculate_light_rating(
        entity,
        lights,
        directional_lights,
        &spatial,
        control,
        translation,
    );

    // movement
    let movement = match velocity.length_squared() {
        v if v < control.low_speed => control.low_speed_mod,
        v if v > control.high_speed => control.high_speed_mod,
        _ => control.medium_speed_mod,
    };

    // exposure
    let closest_wall = calc_closest_wall(translation, spatial, entity);
    let exposure = if closest_wall < control.wall_dist {
        control.wall_mod
    } else {
        0
    };
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

fn calculate_light_rating(
    entity: Entity,
    lights: Query<(
        &GlobalTransform,
        NameOrEntity,
        AnyOf<(&PointLight, &SpotLight)>,
    )>,
    directional_lights: Query<(&GlobalTransform, &DirectionalLight)>,
    spatial: &SpatialQuery,
    control: &AiVisibilityControl,
    translation: Vec3,
) -> u32 {
    let raw_lighting =
        compute_object_lighting(entity, lights, directional_lights, spatial, translation);
    let raw_lighting = (raw_lighting * 100.0).clamp(1.0, 100.0) as u32;
    const LOW_LIGHT_NORM: u32 = 25;
    const MEDIUM_LIGHT_NORM: u32 = 50;
    const HIGH_LIGHT_NORM: u32 = 75;
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
    norm_base + ((raw_lighting - pre_norm_base) as f32 / pre_norm_range as f32) as u32 + norm_range
}

fn compute_object_lighting(
    entity: Entity,
    lights: Query<(
        &GlobalTransform,
        NameOrEntity,
        AnyOf<(&PointLight, &SpotLight)>,
    )>,
    directional_lights: Query<(&GlobalTransform, &DirectionalLight)>,
    spatial: &SpatialQuery,
    translation: Vec3,
) -> f32 {
    let nearby_lights = spatial.shape_intersections(
        &SPHERE,
        translation,
        Quat::IDENTITY,
        &SpatialQueryFilter::from_mask([CollisionLayer::LightSource]),
    );
    let mut lighting = 0.0;
    let filter =
        SpatialQueryFilter::from_mask([CollisionLayer::Opaque]).with_excluded_entities([entity]);
    for (light_transform, light) in directional_lights.iter() {
        let dir = light_transform.rotation().inverse() * Dir3::NEG_Z;
        let hit = spatial.cast_ray(translation, dir, f32::INFINITY, true, &filter);
        if hit.is_some() {
            // Occluded
            continue;
        }
        lighting += estimate_directional_light(light.clone());
    }
    for light in nearby_lights {
        let Ok((light_transform, light_name, (point_light, spot_light))) = lights.get(light) else {
            continue;
        };

        let Ok((dir, len)) = Dir3::new_and_length(light_transform.translation() - translation)
        else {
            // This object *is* the light source
            if let Some(light) = point_light {
                lighting += estimate_point_light(*light, 0.0);
            } else if let Some(light) = spot_light {
                lighting += estimate_spot_light(*light, 0.0, 1.0);
            } else {
                error!("{light_name}: Invalid light type")
            }
            continue;
        };
        let hit = spatial.cast_ray(translation, dir, len, true, &filter);
        if hit.is_some() {
            // Occluded
            continue;
        }
        let light_translation = light_transform.translation();
        let distance_squared = translation.distance_squared(light_translation);
        if let Some(light) = point_light {
            lighting += estimate_point_light(*light, distance_squared);
        } else if let Some(light) = spot_light {
            lighting += estimate_spot_light(*light, 0.0, 1.0);
        } else {
            error!("{light_name}: Invalid light type")
        }
    }
    lighting
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
