use crate::{
    demo::{npc::NPC_FLOAT_HEIGHT, path_corner::PathCorner, target::Target},
    screens::Screen,
    third_party::landmass::Agent,
};
use avian3d::prelude::*;
use bevy::{ecs::relationship::Relationship as _, prelude::*};
use bevy_landmass::{PointSampleDistance3d, Velocity3d as LandmassVelocity, prelude::*};
use bevy_tnua::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        (sync_agent_velocity, set_controller_velocity)
            .chain()
            .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop)
            .before(LandmassSystems::SyncExistence)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(FixedUpdate, set_target_to_quake_target);
}

fn set_controller_velocity(
    mut agent_query: Query<(&mut TnuaController, &Agent)>,
    desired_velocity_query: Query<&AgentDesiredVelocity3d>,
) {
    for (mut controller, agent) in &mut agent_query {
        let Ok(desired_velocity) = desired_velocity_query.get(**agent) else {
            continue;
        };
        let velocity = desired_velocity.velocity();
        let forward = Dir3::try_from(velocity).ok();
        controller.basis(TnuaBuiltinWalk {
            desired_velocity: velocity,
            desired_forward: forward,
            acceleration: 35.0,
            float_height: NPC_FLOAT_HEIGHT,
            ..default()
        });
    }
}

fn sync_agent_velocity(
    mut agent_query: Query<(&LinearVelocity, &Agent)>,
    mut landmass_velocity: Query<&mut LandmassVelocity>,
) {
    for (avian_velocity, agent) in &mut agent_query {
        let Ok(mut landmass_velocity) = landmass_velocity.get_mut(**agent) else {
            error!("Failed to get landmass velocity");
            continue;
        };
        landmass_velocity.velocity = avian_velocity.0;
    }
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
            &PointSampleDistance3d::from_agent_radius(5.0),
        );
        if let Ok(target_point) = target_point {
            *agent_target = AgentTarget3d::Point(target_point.point());
        }
    }
}
