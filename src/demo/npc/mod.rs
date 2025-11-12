use crate::{
    animation::AnimationPlayerAncestor,
    asset_tracking::LoadResource as _,
    collision_layer::CollisionLayer,
    demo::{
        ai::{
            alertness::Alertness,
            hearing::AiSourceBody,
            vision::{view_cone::debug::add_debug_view_cones, visibility::AiVisibility},
        },
        npc::animation::{NpcAnimationState, setup_npc_animations},
        target::TargetBase,
        team::Team,
    },
    link_head::link_head_bone,
    movement::{FloatHeight, SpeedSettings},
    third_party::landmass::AgentOf,
};
use avian_steam_audio::NotSteamAudioCollider;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::prelude::*;
use bevy_tnua::{TnuaAnimatingState, prelude::*};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_trenchbroom::prelude::*;
mod barks;
mod behavior;

mod animation;
mod audio;
pub(crate) mod movement;

pub(super) fn plugin(app: &mut App) {
    app.load_asset::<Gltf>(NPC_GLTF);
    app.add_observer(spawn_npc);
    app.add_plugins((
        movement::plugin,
        animation::plugin,
        audio::plugin,
        behavior::plugin,
        barks::plugin,
    ));
}

const NPC_GLTF: &str = "models/npc.glb";

const NPC_HEIGHT: f32 = 1.6811;
pub(crate) const NPC_RADIUS: f32 = 0.2;
const NPC_FLOAT_HEIGHT: f32 = NPC_HEIGHT / 2.0 + 0.01;
const NPC_WALK_SPEED: f32 = 2.5;
const NPC_RUN_SPEED: f32 = 4.5;

#[point_class(base(TargetBase), model("models/npc.glb"))]
#[derive(Debug)]
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
            NotSteamAudioCollider,
            TnuaController::default(),
            TnuaAvian3dSensorShape(Collider::cylinder(NPC_RADIUS - 0.01, 0.0)),
            ColliderDensity(2_000.0),
            RigidBody::Dynamic,
            TnuaAnimatingState::<NpcAnimationState>::default(),
            AnimationPlayerAncestor,
            FloatHeight(NPC_FLOAT_HEIGHT),
            CollisionLayers::new(
                [CollisionLayer::Default, CollisionLayer::AiVisible],
                LayerMask::ALL,
            ),
            AiVisibility::default(),
            Alertness::default(),
            AiSourceBody,
            SpeedSettings {
                base: NPC_WALK_SPEED,
                run: NPC_RUN_SPEED,
            },
            Team::Bad(0),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    SceneRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(NPC_GLTF))),
                    Transform::from_xyz(0.0, -NPC_FLOAT_HEIGHT, 0.0),
                ))
                .observe(link_head_bone::<Npc>("DEF-head"));
        })
        .observe(add_debug_view_cones)
        .observe(setup_npc_animations);
    commands.spawn((
        Name::new("NPC Agent"),
        Transform::from_translation(Vec3::new(0.0, -NPC_FLOAT_HEIGHT, 0.0)),
        Agent3dBundle {
            agent: default(),
            settings: AgentSettings {
                radius: NPC_RADIUS,
                desired_speed: 0.0,
                max_speed: 0.0,
            },
            archipelago_ref: ArchipelagoRef3d::new(*archipelago),
        },
        ChildOf(npc),
        AgentOf(npc),
        AgentTarget3d::default(),
    ));
}
