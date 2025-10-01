use std::f32::consts::{FRAC_PI_2, PI, TAU};

use avian3d::prelude::*;
use bevy::prelude::*;

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
        assert!(xy_angle > 0.0, "xy_angle must be greater than 0.0");
        assert!(z_angle > 0.0, "z_angle must be greater than 0.0");
        assert!(range > 0.0, "range must be greater than 0.0");
        assert!(
            z_angle < PI,
            "z_angle ({:.1}°) must be less than PI (180°)",
            z_angle.to_degrees()
        );
        assert!(
            xy_angle < TAU,
            "xy_angle ({:.1}°) must be less than TAU (360°)",
            xy_angle.to_degrees()
        );
        let point = |phi, theta| {
            Quat::from_rotation_y(phi) * (Quat::from_rotation_x(theta) * Dir3::NEG_Z * range)
        };
        let convex_hull = |verts| {
            Collider::convex_hull(verts).unwrap_or_else(|| {
            panic!(
                "Failed to create view cone with angle_xy: {:.1}°, angle_z: {:.1}°, range: {} m",
                xy_angle.to_degrees(),
                z_angle.to_degrees(),
                range
            )
        })
        };
        let mut verts = vec![Vec3::ZERO];
        let half_subdiv = 8;
        let phi = xy_angle / 2.0;
        let theta = z_angle / 2.0;
        let (phi, phi_over) = if phi > FRAC_PI_2 {
            (FRAC_PI_2, phi - FRAC_PI_2)
        } else {
            (phi, 0.0)
        };

        for i in -half_subdiv..=half_subdiv {
            for j in -half_subdiv..=half_subdiv {
                let phi = phi * i as f32 / half_subdiv as f32;
                let theta = theta * j as f32 / half_subdiv as f32;
                verts.push(point(phi, theta));
            }
        }
        let front_hull = convex_hull(verts);
        if phi_over == 0.0 {
            return Collider::compound(vec![(Vec3::ZERO, Quat::IDENTITY, front_hull)]);
        }

        let half_phi_over = phi_over / 2.0;
        let mut right_verts = vec![Vec3::ZERO];
        let mut left_verts = vec![Vec3::ZERO];
        for i in 0..=half_subdiv {
            for j in -half_subdiv..=half_subdiv {
                let phi = half_phi_over * i as f32 / half_subdiv as f32 + FRAC_PI_2;
                let theta = theta * j as f32 / half_subdiv as f32;

                right_verts.push(point(phi, theta));
                left_verts.push(point(-phi, theta));
            }
        }
        let right_back_hull = convex_hull(right_verts);
        let left_back_hull = convex_hull(left_verts);

        Collider::compound(vec![
            (Vec3::ZERO, Quat::IDENTITY, front_hull),
            (Vec3::ZERO, Quat::IDENTITY, right_back_hull),
            (Vec3::ZERO, Quat::IDENTITY, left_back_hull),
        ])
    }
}
