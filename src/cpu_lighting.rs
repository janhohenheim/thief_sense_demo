//! Implementation based on
//! - bevy/crates/bevy_pbr/src/render/light.rs: prepare_lights
//! - bevy/crates/bevy_pbr/src/render/pbr_lighting.wgsl: point_light, spot_light, directional_light, getDistanceAttenuation

use std::{f32::consts::FRAC_1_PI, f64::consts::FRAC_PI_4, sync::LazyLock};

use avian3d::{math::PI, prelude::*};
use bevy::{camera::Exposure, math::FloatPow, prelude::*};

use crate::collision_layer::CollisionLayer;

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

const MAX_LIGHT_DISTANCE: f32 = 30.0;
static SPHERE: LazyLock<Collider> = LazyLock::new(|| Collider::sphere(MAX_LIGHT_DISTANCE));

pub(crate) fn estimate_tone_mapped_lighting(
    In(entity): In<Entity>,
    world: &mut World,
) -> Result<f32> {
    let lighting = world.run_system_cached_with(estimate_total_lighting, entity)?;
    let ldr = estimate_tone_mapping(lighting);
    Ok(ldr)
}

fn estimate_total_lighting(
    In(entity): In<Entity>,
    transforms: Query<&GlobalTransform>,
    lights: Query<(
        &GlobalTransform,
        NameOrEntity,
        AnyOf<(&PointLight, &SpotLight)>,
    )>,
    directional_lights: Query<(&GlobalTransform, &DirectionalLight)>,
    spatial: SpatialQuery,
) -> Result<f32> {
    let translation = transforms.get(entity)?.translation();

    let mut lighting = 0.0;
    let filter =
        SpatialQueryFilter::from_mask([CollisionLayer::Opaque]).with_excluded_entities([entity]);
    for (light_transform, light) in directional_lights.iter() {
        let dir = light_transform.back();
        let hit = spatial.cast_ray(translation, dir, f32::INFINITY, true, &filter);
        if hit.is_some() {
            // Occluded
            continue;
        }
        lighting += estimate_directional_light(light.clone());
    }
    let nearby_lights = spatial.shape_intersections(
        &SPHERE,
        translation,
        Quat::IDENTITY,
        &SpatialQueryFilter::from_mask([CollisionLayer::LightSource]),
    );
    for light in nearby_lights {
        let Ok((light_transform, light_name, (point_light, spot_light))) = lights.get(light) else {
            continue;
        };
        let light_transform = light_transform.compute_transform();
        let Ok((dir, len)) = Dir3::new_and_length(light_transform.translation - translation) else {
            // This object *is* the light source
            if let Some(light) = point_light {
                lighting += estimate_point_light(*light, light_transform.translation, translation);
            } else if let Some(light) = spot_light {
                lighting += estimate_spot_light(*light, light_transform, translation);
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
        if let Some(light) = point_light {
            lighting += estimate_point_light(*light, light_transform.translation, translation);
        } else if let Some(light) = spot_light {
            lighting += estimate_spot_light(*light, light_transform, translation);
        } else {
            error!("{light_name}: Invalid light type")
        }
    }
    Ok(lighting)
}

// ignore the object's color by assuming it's perfect white.
// lambertian = albedo / pi = 1 / pi
const MATERIAL_COLOR: f32 = FRAC_1_PI;
// approximate the object as a sphere
const N_DOT_L: f32 = 1.0;

fn estimate_point_light(light: PointLight, light_position: Vec3, point_position: Vec3) -> f32 {
    let distance_squared = light_position.distance_squared(point_position);
    let range_attenuation = get_distance_attenuation(distance_squared, light.range);
    // luminous power to luminous intensity
    let intensity = light.intensity / (4.0 * PI);
    let color_intensity =
        Vec3::from_array(light.color.to_linear().to_f32_array_no_alpha()) * intensity;
    let luminance = luminance(color_intensity);

    MATERIAL_COLOR * luminance * range_attenuation * N_DOT_L
}

fn estimate_spot_light(light: SpotLight, light_transform: Transform, point_position: Vec3) -> f32 {
    let point_light_equivalent = PointLight {
        color: light.color,
        intensity: light.intensity,
        range: light.range,
        ..default()
    };
    let point_light_contribution = estimate_point_light(
        point_light_equivalent,
        light_transform.translation,
        point_position,
    );

    let light_to_point = (point_position - light_transform.translation).normalize();
    let cos_angle = light_transform.forward().dot(light_to_point);

    let cos_inner = light.inner_angle.cos();
    let cos_outer = light.outer_angle.cos();
    let spot_scale = 1.0 / (cos_inner - cos_outer).max(1e-4);
    let spot_offset = -cos_outer * spot_scale;

    let attenuation = (cos_angle * spot_scale + spot_offset).clamp(0.0, 1.0);
    let spot_attenuation = attenuation * attenuation;

    point_light_contribution * spot_attenuation
}

fn estimate_directional_light(light: DirectionalLight) -> f32 {
    let color_intensity =
        Vec3::from_array(light.color.to_linear().to_f32_array_no_alpha()) * light.illuminance;
    let luminance = luminance(color_intensity);
    MATERIAL_COLOR * luminance * N_DOT_L
}

fn get_distance_attenuation(distance_square: f32, range: f32) -> f32 {
    let factor = distance_square / range.squared();
    let smooth_factor = (1.0 - factor.squared()).clamp(0.0, 1.0);
    let attenuation = smooth_factor.squared();
    attenuation / distance_square.max(0.0001)
}

fn estimate_tone_mapping(light_contribution: f32) -> f32 {
    let exposure = Exposure::default().exposure();
    let hdr = light_contribution * exposure;
    let ldr = custom_tone_mapping(hdr);
    ldr.clamp(0.0, 1.0)
}

fn luminance(color: Vec3) -> f32 {
    Color::linear_rgb(color.x, color.y, color.z).luminance()
}

fn reinhard_ext(color: f32) -> f32 {
    const MAX_WHITE: f32 = 4.0;
    let numerator = color * (1.0 + (color / MAX_WHITE.squared()));
    numerator / (1.0 + color)
}

/// A custom-tuned tonemap that preserves contrast in dark areas and compresses bright areas.
/// This is needed because a stealth system cares much more about dark details than bright ones.
fn custom_tone_mapping(hdr: f32) -> f32 {
    const DARK_STOP: f32 = 0.2;
    const REINHARD_START: f32 = 0.25;
    const DARK_BOOST: f32 = 3.5;

    if hdr <= DARK_STOP {
        // boost contrast in dark areas
        (hdr / DARK_STOP).powf(1.0 / DARK_BOOST) * DARK_STOP
    } else if hdr <= REINHARD_START {
        // Linear zone
        let t = (hdr - DARK_STOP) / (REINHARD_START - DARK_STOP);
        DARK_STOP + t * (REINHARD_START - DARK_STOP)
    } else {
        // Apply Reinhard to the excess above the threshold
        let excess = hdr - REINHARD_START;
        let reinhard_result = reinhard_ext(excess);
        REINHARD_START + reinhard_result
    }
}
