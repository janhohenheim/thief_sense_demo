use crate::{
    animation::AnimationPlayerAncestor, asset_tracking::LoadResource as _,
    demo::npc::animation::NpcAnimationState, third_party::landmass::AgentOf,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::prelude::*;
use bevy_tnua::{TnuaAnimatingState, prelude::*};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_trenchbroom::prelude::*;

mod animation;
mod movement;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Npc>();
    app.load_asset::<Gltf>(NPC_GLTF);
    app.add_observer(spawn_npc);
    app.add_plugins((movement::plugin, animation::plugin));
}

const NPC_GLTF: &str = "models/npc.glb";

const NPC_HEIGHT: f32 = 1.6811;
const NPC_RADIUS: f32 = 0.2;
const NPC_FLOAT_HEIGHT: f32 = NPC_HEIGHT / 2.0 + 0.01;

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
            Collider::capsule(NPC_RADIUS, NPC_HEIGHT - NPC_RADIUS * 2.0),
            TnuaController::default(),
            TnuaAvian3dSensorShape(Collider::cylinder(NPC_RADIUS - 0.01, 0.0)),
            ColliderDensity(2_000.0),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
            TnuaAnimatingState::<NpcAnimationState>::default(),
            AnimationPlayerAncestor,
        ))
        .with_child((
            SceneRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(NPC_GLTF))),
            Transform::from_xyz(0.0, -NPC_FLOAT_HEIGHT, 0.0),
        ));
    commands.spawn((
        Name::new("NPC Agent"),
        Transform::from_translation(Vec3::new(0.0, -NPC_FLOAT_HEIGHT, 0.0)),
        Agent3dBundle {
            agent: default(),
            settings: AgentSettings {
                radius: NPC_RADIUS,
                desired_speed: 5.0,
                max_speed: 7.0,
            },
            archipelago_ref: ArchipelagoRef3d::new(*archipelago),
        },
        ChildOf(npc),
        AgentOf(npc),
        AgentTarget3d::default(),
    ));
}
