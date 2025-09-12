use crate::{asset_tracking::LoadResource as _, screens::Screen};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{TargetReachedCondition, Velocity3d as LandmassVelocity, prelude::*};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_trenchbroom::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Npc>();
    app.load_asset::<Gltf>(MODEL_PATH);
    app.add_observer(spawn_npc);

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
    app.add_systems(
        RunFixedMainLoop,
        update_agent_target.in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
    );
}

const MODEL_PATH: &'static str = "models/npc.glb";

#[point_class(model(MODEL_PATH))]
pub struct Npc;

fn spawn_npc(
    trigger: Trigger<OnAdd, Npc>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    archipelago: Single<Entity, With<Archipelago3d>>,
) {
    let npc = trigger.target();
    commands
        .entity(npc)
        .insert((
            Collider::capsule(0.5, 0.5),
            TnuaController::default(),
            TnuaAvian3dSensorShape(Collider::cylinder(0.5 - 0.01, 0.0)),
            ColliderDensity(2_000.0),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
        ))
        .with_child(SceneRoot(
            assets.load(GltfAssetLabel::Scene(0).from_asset(MODEL_PATH)),
        ));
    commands.spawn((
        Name::new("NPC Agent"),
        Transform::from_translation(Vec3::new(0.0, -1.0, 0.0)),
        Agent3dBundle {
            agent: default(),
            settings: AgentSettings {
                radius: 0.5,
                desired_speed: 7.0,
                max_speed: 8.0,
            },
            archipelago_ref: ArchipelagoRef3d::new(*archipelago),
        },
        TargetReachedCondition::Distance(Some(1.0)),
        ChildOf(npc),
        AgentOf(npc),
        AgentTarget3d::default(),
    ));
}

fn update_agent_target(mut agents: Query<&mut AgentTarget3d>) {
    for mut target in &mut agents {
        *target = AgentTarget3d::Point(Vec3::new(3.0, 0.0, 0.0));
    }
}

#[derive(Component, Deref, Debug, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = Agent)]
struct AgentOf(Entity);

#[derive(Component, Deref, Debug, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = AgentOf)]
struct Agent(Entity);

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
