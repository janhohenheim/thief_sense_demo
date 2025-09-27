use crate::{
    animation::AnimationPlayerAncestor,
    asset_tracking::LoadResource as _,
    demo::{
        link_head::link_head_bone,
        npc::{animation::NpcAnimationState, sense::SenseTimer},
        target::TargetBase,
    },
    movement::FloatHeight,
    third_party::landmass::AgentOf,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::prelude::*;
use bevy_tnua::{TnuaAnimatingState, prelude::*};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_trenchbroom::prelude::*;

mod animation;
mod look;
mod movement;
mod sense;
mod view_cone;

pub(super) fn plugin(app: &mut App) {
    app.load_asset::<Gltf>(NPC_GLTF);
    app.add_observer(spawn_npc);
    app.add_plugins((
        movement::plugin,
        animation::plugin,
        view_cone::plugin,
        look::plugin,
        sense::plugin,
    ));
}

const NPC_GLTF: &str = "models/npc.glb";

const NPC_HEIGHT: f32 = 1.6811;
pub(crate) const NPC_RADIUS: f32 = 0.2;
const NPC_FLOAT_HEIGHT: f32 = NPC_HEIGHT / 2.0 + 0.01;
const NPC_MAX_SPEED: f32 = 5.0;
const NPC_WALK_SPEED: f32 = 2.5;

#[point_class(base(TargetBase), model("models/npc.glb"))]
pub(crate) struct Npc;

fn spawn_npc(
    trigger: On<Add, Npc>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    archipelago: Single<Entity, With<Archipelago3d>>,
) {
    let npc = trigger.entity;
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
            FloatHeight(NPC_FLOAT_HEIGHT),
            SenseTimer::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    SceneRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(NPC_GLTF))),
                    Transform::from_xyz(0.0, -NPC_FLOAT_HEIGHT, 0.0),
                ))
                .observe(link_head_bone::<Npc>("DEF-head"));
        });
    commands.spawn((
        Name::new("NPC Agent"),
        Transform::from_translation(Vec3::new(0.0, -NPC_FLOAT_HEIGHT, 0.0)),
        Agent3dBundle {
            agent: default(),
            settings: AgentSettings {
                radius: NPC_RADIUS,
                desired_speed: NPC_WALK_SPEED,
                max_speed: NPC_MAX_SPEED,
            },
            archipelago_ref: ArchipelagoRef3d::new(*archipelago),
        },
        ChildOf(npc),
        AgentOf(npc),
        AgentTarget3d::default(),
    ));
}
