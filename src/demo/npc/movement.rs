use crate::screens::Screen;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{Velocity3d as LandmassVelocity, prelude::*};
use bevy_tnua::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Agent>();
    app.register_type::<AgentOf>();
    app.add_systems(
        RunFixedMainLoop,
        (sync_agent_velocity, set_controller_velocity)
            .chain()
            .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop)
            .before(LandmassSystemSet::SyncExistence)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component, Deref, Debug, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = Agent)]
pub(crate) struct AgentOf(pub(crate) Entity);

#[derive(Component, Deref, Debug, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = AgentOf)]
pub(crate) struct Agent(Entity);

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
            float_height: 1.0,
            ..default()
        });
    }
}

fn sync_agent_velocity(mut agent_query: Query<(&LinearVelocity, &mut LandmassVelocity)>) {
    for (avian_velocity, mut landmass_velocity) in &mut agent_query {
        landmass_velocity.velocity = avian_velocity.0;
    }
}
