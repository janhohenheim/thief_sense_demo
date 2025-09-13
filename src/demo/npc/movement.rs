use crate::{demo::npc::NPC_FLOAT_HEIGHT, screens::Screen, third_party::landmass::Agent};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{Velocity3d as LandmassVelocity, prelude::*};
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
