use avian3d::prelude::*;
use bevy::prelude::*;

use crate::demo::ai::view_cone::{
    ViewCone, ViewConeFlags, ViewCones, VisibilityAcuities, VisibilityAcuity,
    collider::ViewCone as _,
};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

impl FromWorld for ViewCones {
    fn from_world(_world: &mut World) -> Self {
        Self(vec![
            // Close up near perfect vision, high alert
            ViewCone {
                collider: Collider::view_cone(170.0_f32.to_radians(), 170.0_f32.to_radians(), 1.5),
                flags: ViewConeFlags::Active
                    | ViewConeFlags::LowLight
                    | ViewConeFlags::NoAlert0
                    | ViewConeFlags::NoAlert1,
                acuity: 1500.0,
            },
            // Close up near perfect vision, low alert
            ViewCone {
                collider: Collider::view_cone(50.0_f32.to_radians(), 100.0_f32.to_radians(), 1.5),
                flags: ViewConeFlags::Active
                    | ViewConeFlags::LowLight
                    | ViewConeFlags::NoAlert2
                    | ViewConeFlags::NoAlert3,
                acuity: 1500.0,
            },
            ViewCone {
                collider: Collider::view_cone(170.0_f32.to_radians(), 17.0_f32.to_radians(), 1.5),
                flags: ViewConeFlags::Active | ViewConeFlags::NoAlert2 | ViewConeFlags::NoAlert3,
                acuity: 1500.0,
            },
            // Near cone perfect vision
            ViewCone {
                collider: Collider::view_cone(60.0_f32.to_radians(), 90.0_f32.to_radians(), 3.0),
                flags: ViewConeFlags::Active,
                acuity: 200.0,
            },
            /*
            // Round the back magic vision
            ViewCone {
                collider: Collider::view_cone(320.0_f32.to_radians(), 90.0_f32.to_radians(), 1.8),
                flags: ViewConeFlags::Active
                    | ViewConeFlags::NoAlert0
                    | ViewConeFlags::NoAlert1
                    | ViewConeFlags::Omni,
                acuity: 70.0,
            }, */
            // Normal near binocular vision
            ViewCone {
                collider: Collider::view_cone(120.0_f32.to_radians(), 90.0_f32.to_radians(), 6.7),
                flags: ViewConeFlags::Active,
                acuity: 120.0,
            },
            // Mid-range sight
            /*
            ViewCone {
                collider: Collider::view_cone(150.0_f32.to_radians(), 70.0_f32.to_radians(), 10.6),
                flags: ViewConeFlags::Active,
                acuity: 80.0,
            },
            // Long range and wide range peripheral vision
            ViewCone {
                collider: Collider::view_cone(230.0_f32.to_radians(), 70.0_f32.to_radians(), 10.6),
                flags: ViewConeFlags::Active | ViewConeFlags::Periph,
                acuity: 18.2,
            },
            // Long range and high Z
            ViewCone {
                collider: Collider::view_cone(230.0_f32.to_radians(), 110.0_f32.to_radians(), 24.3),
                flags: ViewConeFlags::Active | ViewConeFlags::Periph,
                acuity: 12.1,
            },
            */
        ])
    }
}

const VISIBILITY_ACUITIES: VisibilityAcuities = VisibilityAcuities {
    normal: VisibilityAcuity {
        lighting: 1.0,
        movement: 1.0,
        exposure: 1.0,
    },
    periphery: VisibilityAcuity {
        lighting: 0.3,
        movement: 3.0,
        exposure: 1.0,
    },
    omnidirectional: VisibilityAcuity {
        lighting: 0.8,
        movement: 1.4,
        exposure: 1.2,
    },
    light: VisibilityAcuity {
        lighting: 1.0,
        movement: 0.0,
        exposure: 0.0,
    },
    movement: VisibilityAcuity {
        lighting: 0.0,
        movement: 5.0,
        exposure: 0.0,
    },
    low_light: VisibilityAcuity {
        lighting: 6.0,
        movement: 1.0,
        exposure: 1.0,
    },
};
