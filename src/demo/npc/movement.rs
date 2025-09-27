use crate::{
    demo::{
        npc::{NPC_FLOAT_HEIGHT, NPC_RADIUS},
        path_corner::PathCorner,
        target::Target,
    },
    screens::Screen,
    third_party::landmass::Agent,
};
use avian3d::prelude::*;
use bevy::{ecs::relationship::Relationship as _, prelude::*};
use bevy_landmass::{PointSampleDistance3d, Velocity3d as LandmassVelocity, prelude::*};
use bevy_tnua::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, set_target_to_quake_target);
}

fn set_target_to_quake_target(
    controllers: Query<(&Target, &Agent)>,
    mut agents: Query<&mut AgentTarget3d>,
    path_corners: Query<(), With<PathCorner>>,
    transforms: Query<&Transform>,
    archipelago: Single<&Archipelago3d>,
) {
    for (target, agent) in &controllers {
        let Ok(mut agent_target) = agents.get_mut(**agent) else {
            error!("Failed to get agent target");
            continue;
        };
        if !path_corners.contains(target.get()) {
            continue;
        }
        let Ok(transform) = transforms.get(target.get()) else {
            error!("Failed to get transform for target");
            continue;
        };
        let target_point = archipelago.sample_point(
            transform.translation,
            &PointSampleDistance3d::from_agent_radius(NPC_RADIUS),
        );
        if let Ok(target_point) = target_point {
            *agent_target = AgentTarget3d::Point(target_point.point());
        }
    }
}
