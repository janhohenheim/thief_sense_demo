use std::f32::consts::{FRAC_PI_2, TAU};

use avian3d::prelude::*;
use bevy::{math::ops::sin_cos, prelude::*};

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
        // The following is adapted from Bevy's implementation of building a mesh from an ellipse
        const RESOLUTION: usize = 8;
        // Add pi/2 so that there is a vertex at the top (sin is 1.0 and cos is 0.0)
        let start_angle = FRAC_PI_2;
        let step = TAU / RESOLUTION as f32;

        let tip = Vec3::ZERO;
        let mut points = vec![tip];
        for i in 0..RESOLUTION {
            // Compute vertex position at angle theta
            let theta = start_angle + i as f32 * step;
            let (sin, cos) = sin_cos(theta);
            let x = cos * half_width;
            let z = sin * half_height;

            points.push(Vec3::new(x, -height, z));
        }
        Collider::convex_hull(points).unwrap()
    }
}
