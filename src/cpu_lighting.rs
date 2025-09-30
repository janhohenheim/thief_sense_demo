//! Implementation based on
//! - bevy/crates/bevy_pbr/src/render/light.rs: prepare_lights
//! - bevy/crates/bevy_pbr/src/render/pbr_lighting.wgsl: point_light, spot_light, directional_light, getDistanceAttenuation

use std::f32::consts::FRAC_1_PI;

use bevy::{camera::Exposure, math::FloatPow, prelude::*};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

// ignore the object's color by assuming it's perfect white.
const MATERIAL_COLOR: f32 = FRAC_1_PI;
// approximate the object as a sphere
const N_DOT_L: f32 = 1.0;

pub(crate) fn estimate_point_light(
    light: PointLight,
    light_position: Vec3,
    point_position: Vec3,
) -> f32 {
    let distance_squared = light_position.distance_squared(point_position);
    let range_attenuation = get_distance_attenuation(distance_squared, light.range);
    let color_intensity =
        Vec3::from_array(light.color.to_linear().to_f32_array_no_alpha()) * light.intensity;
    let luminance = luminance(color_intensity);

    MATERIAL_COLOR * luminance * range_attenuation * N_DOT_L
}

pub(crate) fn estimate_spot_light(
    light: SpotLight,
    light_transform: Transform,
    point_position: Vec3,
) -> f32 {
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

pub(crate) fn estimate_directional_light(light: DirectionalLight) -> f32 {
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

pub(crate) fn estimate_tone_mapping(light_contribution: f32) -> f32 {
    let exposure = Exposure::default().exposure();
    let hdr = light_contribution * exposure;
    let ldr = reinhard_ext(hdr);
    ldr.clamp(0.0, 1.0)
}

fn luminance(color: Vec3) -> f32 {
    Color::linear_rgb(color.x, color.y, color.z).luminance()
}

fn reinhard_ext(color: f32) -> f32 {
    const MAX_WHITE: f32 = 11.0;
    let numerator = color * (1.0 + (color / MAX_WHITE.squared()));
    return numerator / (1.0 + color);
}
