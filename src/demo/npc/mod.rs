use crate::{asset_tracking::LoadResource as _, demo::npc::movement::AgentOf};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{TargetReachedCondition, prelude::*};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_trenchbroom::prelude::*;

mod animation;
mod movement;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Npc>();
    app.load_asset::<Gltf>(MODEL_PATH);
    app.add_observer(spawn_npc);
    app.add_plugins((movement::plugin, animation::plugin));
}

const MODEL_PATH: &str = "models/npc.glb";

#[point_class(model(MODEL_PATH))]
pub(crate) struct Npc;

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
