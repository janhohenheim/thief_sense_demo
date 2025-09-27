use crate::{screens::Screen, third_party::landmass::Agent};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{Velocity3d as LandmassVelocity, prelude::*};
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
}

#[derive(Component)]
pub(crate) struct FloatHeight(pub(crate) f32);

fn set_controller_velocity(
    mut agent_query: Query<(&mut TnuaController, &Agent, &FloatHeight)>,
    desired_velocity_query: Query<&AgentDesiredVelocity3d>,
) {
    for (mut controller, agent, float_height) in &mut agent_query {
        let Ok(desired_velocity) = desired_velocity_query.get(**agent) else {
            continue;
        };
        let velocity = desired_velocity.velocity();
        let forward = Dir3::try_from(velocity).ok();
        controller.basis(TnuaBuiltinWalk {
            desired_velocity: velocity,
            desired_forward: forward,
            acceleration: 35.0,
            float_height: float_height.0,
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
