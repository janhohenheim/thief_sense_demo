use std::{f32::consts::TAU, iter};

use avian3d::prelude::*;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()));
}

pub(crate) trait EllipticCone {
    /// Like [`Collider::cone`], but using an elliptical base. `half_width` is along the x-axis, `half_height` is along the z-axis.
    /// The origin is at the tip of the cone.
    fn elliptic_cone(half_width: f32, half_height: f32, height: f32) -> Collider;
}

impl EllipticCone for Collider {
    fn elliptic_cone(half_width: f32, half_height: f32, height: f32) -> Collider {
        let ellipse = Ellipse::new(half_width, half_height)
            .mesh()
            .resolution(8)
            .build();
        let base = ellipse
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .into_iter()
            .copied()
            .map(Vec3::from_array)
            .map(|v| Quat::from_rotation_x(TAU / 4.0) * v)
            .map(|v| v - Vec3::Y * height);
        let tip = Vec3::ZERO;
        let points = iter::once(tip).chain(base).collect();
        Collider::convex_hull(points).unwrap()
    }
}
