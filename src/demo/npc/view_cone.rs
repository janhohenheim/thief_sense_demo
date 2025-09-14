use std::f32::consts::TAU;

use crate::third_party::avian::EllipticCone as _;
use avian3d::prelude::*;
use bevy::{prelude::*, scene::SceneInstanceReady};

pub(super) fn plugin(_app: &mut App) {}

pub(crate) fn spawn_view_cones(
    trigger: Trigger<SceneInstanceReady>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut commands: Commands,
) {
    for child in children.iter_descendants(trigger.target()) {
        let Ok(name) = names.get(child) else {
            continue;
        };
        if name.as_str() != "DEF-head" {
            continue;
        }
        let view_cone = Collider::elliptic_cone(0.4, 0.8, 1.5);
        let sphere = Collider::sphere(0.2);

        commands
            .entity(child)
            .with_child((
                view_cone,
                Transform::from_rotation(Quat::from_rotation_x(-TAU / 4.0)),
            ))
            .with_child(sphere);

        break;
    }
}
