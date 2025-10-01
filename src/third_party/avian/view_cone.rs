use avian3d::prelude::*;
use bevy::prelude::*;

pub(crate) trait EllipticCone {
    /// Like [`Collider::cone`], but using an elliptical base. `half_width` is along the x-axis, `half_height` is along the z-axis.
    /// The origin is at the tip of the cone.
    fn view_cone(phi: f32, theta: f32, r: f32) -> Collider;
}

impl EllipticCone for Collider {
    fn view_cone(xy_angle: f32, z_angle: f32, range: f32) -> Collider {
        let point = |phi: f32, theta: f32| -> Vec3 {
            Quat::from_rotation_x(phi) * Quat::from_rotation_y(theta) * Dir3::NEG_Z * range
        };
        let mut verts = vec![Vec3::ZERO]; // idx 0
        let half_subdiv = 3;
        let phi = xy_angle / 2.0;
        let theta = z_angle / 2.0;
        let phi_step = phi / half_subdiv as f32;
        let theta_step = theta / half_subdiv as f32;
        // intersection between pyramid and sphere
        //
        // corners
        verts.push(point(-phi, -theta)); // idx 1
        verts.push(point(phi, -theta)); // idx 2
        verts.push(point(-phi, theta)); // idx 3
        verts.push(point(phi, theta)); // idx 4

        let mut indices = Vec::new();

        // tris from tip to corner to point on rim
        //  -theta: 1 -> 2
        let mut prev_idx = 1;
        for i in -(half_subdiv - 1)..half_subdiv {
            let phi = i as f32 * phi_step;
            let theta = -theta;
            let idx = verts.len() as u32;
            verts.push(point(phi, theta));
            indices.push([0, prev_idx, idx]);
            prev_idx = idx;
        }
        indices.push([0, prev_idx, 2]);

        //  +theta: 3 -> 4
        prev_idx = 3;
        for i in -(half_subdiv - 1)..half_subdiv {
            let phi = i as f32 * phi_step;
            let theta = theta;
            let idx = verts.len() as u32;
            verts.push(point(phi, theta));
            indices.push([0, idx, prev_idx]);
            prev_idx = idx;
        }
        indices.push([0, 4, prev_idx]);

        // -phi: 1 -> 3
        prev_idx = 1;
        for i in -(half_subdiv - 1)..half_subdiv {
            let phi = -phi;
            let theta = i as f32 * theta_step;
            let idx = verts.len() as u32;
            verts.push(point(phi, theta));
            indices.push([0, idx, prev_idx]);
            prev_idx = idx;
        }
        indices.push([0, 3, prev_idx]);

        // +phi: 2 -> 4
        prev_idx = 2;
        for i in -(half_subdiv - 1)..half_subdiv {
            let phi = phi;
            let theta = i as f32 * theta_step;
            let idx = verts.len() as u32;
            verts.push(point(phi, theta));
            indices.push([0, prev_idx, idx]);
            prev_idx = idx;
        }
        indices.push([0, prev_idx, 4]);

        /*
        // sphere
        for i in -half_subdiv..half_subdiv {
            for j in -half_subdiv..half_subdiv {
                let phi = phi * i as f32 / half_subdiv as f32;
                let theta = theta * j as f32 / half_subdiv as f32;
                let dir = Quat::from_rotation_x(phi) * Quat::from_rotation_y(theta) * Dir3::NEG_Z;
                let point = dir * range;
                verts.push(point);
            }
        } */

        Collider::trimesh(verts, indices)
    }
}
