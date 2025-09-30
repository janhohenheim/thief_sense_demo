//! Yoinked from <https://github.com/bevyengine/bevy/pull/18207>

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
    },
};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

pub(crate) trait SolidColorEnvironmentMapLight {
    fn solid_color(assets: &mut Assets<Image>, color: Color) -> Self;
}
impl SolidColorEnvironmentMapLight for EnvironmentMapLight {
    /// An environment map with a uniform color, useful for uniform ambient lighting.
    fn solid_color(assets: &mut Assets<Image>, color: Color) -> Self {
        hemispherical_gradient(assets, color, color, color)
    }
}

/// An environment map with a hemispherical gradient, fading between the sky and ground colors
/// at the horizon. Useful as a very simple 'sky'.
fn hemispherical_gradient(
    assets: &mut Assets<Image>,
    top_color: Color,
    mid_color: Color,
    bottom_color: Color,
) -> EnvironmentMapLight {
    let handle = assets.add(hemispherical_gradient_cubemap(
        top_color,
        mid_color,
        bottom_color,
    ));

    EnvironmentMapLight {
        diffuse_map: handle.clone(),
        specular_map: handle,
        ..Default::default()
    }
}

fn hemispherical_gradient_cubemap(
    top_color: Color,
    mid_color: Color,
    bottom_color: Color,
) -> Image {
    let top_color: Srgba = top_color.into();
    let mid_color: Srgba = mid_color.into();
    let bottom_color: Srgba = bottom_color.into();
    Image {
        texture_view_descriptor: Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        }),
        ..Image::new(
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 6,
            },
            TextureDimension::D2,
            [
                mid_color,
                mid_color,
                top_color,
                bottom_color,
                mid_color,
                mid_color,
            ]
            .into_iter()
            .flat_map(Srgba::to_u8_array)
            .collect(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        )
    }
}
