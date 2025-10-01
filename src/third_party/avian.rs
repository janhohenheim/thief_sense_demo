use std::f32::consts::{FRAC_PI_2, PI, TAU};

use avian3d::prelude::*;
use bevy::{
    math::{
        bounding::{BoundingSphere, RayCast3d},
        ops::sin_cos,
    },
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default());
}

pub(crate) trait EllipticCone {
    /// Like [`Collider::cone`], but using an elliptical base. `half_width` is along the x-axis, `half_height` is along the z-axis.
    /// The origin is at the tip of the cone.
    fn view_cone(phi: f32, theta: f32, r: f32) -> Collider;
}

impl EllipticCone for Collider {
    fn view_cone(xy_angle: f32, z_angle: f32, range: f32) -> Collider {
        let mut verts = vec![Vec3::ZERO];
        let half_subdiv = 8;
        let phi = xy_angle / 2.0;
        let theta = z_angle / 2.0;
        for i in -half_subdiv..half_subdiv {
            for j in -half_subdiv..half_subdiv {
                let phi = phi * i as f32 / half_subdiv as f32;
                let theta = theta * j as f32 / half_subdiv as f32;
                let dir = Quat::from_rotation_x(phi) * Quat::from_rotation_y(theta) * Dir3::NEG_Z;
                let point = dir * range;
                verts.push(point);
            }
        }
        Collider::convex_hull(verts).unwrap_or_else(|| {
            panic!(
                "Failed to create rounded cone with angle_xy: {:.1}°, angle_z: {:.1}°, slant: {} m",
                xy_angle.to_degrees(),
                z_angle.to_degrees(),
                range
            )
        })
    }
}
