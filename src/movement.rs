use crate::{screens::Screen, third_party::landmass::Agent};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{Velocity3d as LandmassVelocity, prelude::*};
use bevy_tnua::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedPreUpdate,
        (
            set_desired_velocity,
            sync_agent_velocity,
            set_controller_velocity,
        )
            .chain()
            .before(LandmassSystems::SyncExistence)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.register_required_components::<Agent, IsRunning>();
    app.register_required_components::<Agent, SpeedSettings>();
}

#[derive(Component)]
pub(crate) struct FloatHeight(pub(crate) f32);

#[derive(Component, Default, Deref, DerefMut)]
pub(crate) struct IsRunning(pub(crate) bool);

#[derive(Component, Default, Clone, Copy)]
pub(crate) struct SpeedSettings {
    pub(crate) base: f32,
    pub(crate) run: f32,
}

fn set_desired_velocity(
    mut npcs: Query<
        (&SpeedSettings, &IsRunning, &Agent),
        Or<(Changed<SpeedSettings>, Changed<IsRunning>)>,
    >,
    mut agents: Query<&mut AgentSettings>,
) {
    for (speed_settings, is_running, agent) in &mut npcs {
        let Ok(mut agent_settings) = agents.get_mut(agent.get()) else {
            continue;
        };
        agent_settings.desired_speed = if is_running.0 {
            speed_settings.run
        } else {
            speed_settings.base
        };
        agent_settings.max_speed = if is_running.0 {
            speed_settings.run
        } else {
            speed_settings.run + 0.1
        };
    }
}

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
