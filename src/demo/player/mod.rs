use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::prelude::*;
use bevy_tnua::{TnuaAnimatingState, prelude::*};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_trenchbroom::prelude::*;

mod animation;
mod movement;

use crate::{
    animation::AnimationPlayerAncestor,
    asset_tracking::LoadResource as _,
    demo::{link_head::link_head_bone, player::animation::PlayerAnimationState},
    movement::FloatHeight,
    third_party::{avian::CollisionLayer, landmass::AgentOf},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_player);
    app.add_plugins((animation::plugin, movement::plugin));
    app.load_asset::<Gltf>(PLAYER_GLTF);
}

const PLAYER_GLTF: &str = "models/npc.glb";

const PLAYER_HEIGHT: f32 = 1.6811;
pub(crate) const PLAYER_RADIUS: f32 = 0.2;
const PLAYER_FLOAT_HEIGHT: f32 = PLAYER_HEIGHT / 2.0 + 0.01;
const PLAYER_WALK_SPEED: f32 = 2.5;
const PLAYER_RUN_SPEED: f32 = 5.0;

#[point_class(model("models/npc.glb"))]
pub(crate) struct Player;

fn spawn_player(
    trigger: On<Add, Player>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    archipelago: Single<Entity, With<Archipelago3d>>,
) {
    let player = trigger.entity;
    commands
        .entity(player)
        .insert((
            Collider::capsule(PLAYER_RADIUS, PLAYER_HEIGHT - PLAYER_RADIUS * 2.0),
            TnuaController::default(),
            TnuaAvian3dSensorShape(Collider::cylinder(PLAYER_RADIUS - 0.01, 0.0)),
            ColliderDensity(2_000.0),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
            TnuaAnimatingState::<PlayerAnimationState>::default(),
            CollisionLayers::new(
                [CollisionLayer::Default, CollisionLayer::AiVisible],
                CollisionLayer::Default,
            ),
            AnimationPlayerAncestor,
            FloatHeight(PLAYER_FLOAT_HEIGHT),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    SceneRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(PLAYER_GLTF))),
                    Transform::from_xyz(0.0, -PLAYER_FLOAT_HEIGHT, 0.0),
                ))
                .observe(link_head_bone::<Player>("DEF-head"));
        });
    commands.spawn((
        Name::new("Player Agent"),
        Transform::from_translation(Vec3::new(0.0, -PLAYER_FLOAT_HEIGHT, 0.0)),
        Agent3dBundle {
            agent: default(),
            settings: AgentSettings {
                radius: PLAYER_RADIUS,
                desired_speed: PLAYER_WALK_SPEED,
                max_speed: PLAYER_WALK_SPEED + 2.0,
            },
            archipelago_ref: ArchipelagoRef3d::new(*archipelago),
        },
        ChildOf(player),
        AgentOf(player),
        AgentTarget3d::default(),
    ));
}
