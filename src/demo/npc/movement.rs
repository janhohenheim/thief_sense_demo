use crate::{
    demo::{npc::NPC_FLOAT_HEIGHT, target::Target},
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
            .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop)
            .before(LandmassSystemSet::SyncExistence)
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
        // Hack: due to a bug, landmass slows down on corners, so we set the velocity manually.
        let velocity = desired_velocity.velocity().normalize_or_zero() * 5.0;
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

fn sync_agent_velocity(mut agent_query: Query<(&LinearVelocity, &mut LandmassVelocity)>) {
    for (avian_velocity, mut landmass_velocity) in &mut agent_query {
        landmass_velocity.velocity = avian_velocity.0;
    }
}

fn set_target_to_quake_target(
    mut npcs: Query<(&Target, &mut AgentTarget3d)>,
    transforms: Query<&Transform>,
    archipelago: Single<&Archipelago3d>,
) {
    for (target, mut agent_target) in &mut npcs {
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
