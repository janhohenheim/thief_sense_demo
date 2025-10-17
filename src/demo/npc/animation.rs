//! NPC animation handling.

use std::time::Duration;

use bevy::{
    animation::{AnimationEvent, AnimationTarget, AnimationTargetId},
    prelude::*,
};
use bevy_tnua::{TnuaAnimatingState, TnuaAnimatingStateDirective, prelude::*};

use crate::{
    AppSystems,
    animation::AnimationPlayers,
    demo::npc::{NPC_GLTF, NPC_MAX_SPEED, NPC_WALK_SPEED},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        play_animations
            .run_if(in_state(Screen::Gameplay).and(resource_exists::<NpcAnimations>))
            .in_set(AppSystems::Update),
    );
    app.add_observer(setup_npc_animations);
}

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
struct NpcAnimations {
    graph: AnimationGraphHandle,
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    run: AnimationNodeIndex,
    left_foot: AnimationTargetId,
    right_foot: AnimationTargetId,
}

fn setup_npc_animations(
    trigger: On<Add, AnimationPlayers>,
    q_anim_players: Query<&AnimationPlayers>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    targets: Query<(NameOrEntity, &AnimationTarget)>,
    mut clips: ResMut<Assets<AnimationClip>>,
    gltfs: Res<Assets<Gltf>>,
    children: Query<&Children>,
    animations: Option<Res<NpcAnimations>>,
) {
    let gltf = gltfs.get(assets.load(NPC_GLTF).id()).unwrap();
    let anim_players = q_anim_players.get(trigger.entity).unwrap();
    let get_target_id = |name| {
        get_target_id(name, trigger.entity, children, targets)
            .unwrap_or_else(|| panic!("failed to find bone {name}"))
    };

    for anim_player in anim_players.iter() {
        let graph = if let Some(ref animations) = animations {
            animations.graph.clone()
        } else {
            let (graph, indices) = AnimationGraph::from_clips(
                ["Idle_Loop", "Walk_Loop", "Sprint_Loop"]
                    .map(|name| gltf.named_animations[name].clone()),
            );
            let [idle_index, walk_index, run_index] = indices.as_slice() else {
                panic!("Failed to map animation indices")
            };

            let (left_foot_entity, left_foot_id) = get_target_id("DEF-foot.L");
            let (right_foot_entity, right_foot_id) = get_target_id("DEF-foot.R");
            let frame_time = |frame: u32| (frame - 1) as f32 / 24.0;

            let walk_clip = get_clip(*walk_index, &graph, &mut clips);
            walk_clip.add_event_to_target(left_foot_id, frame_time(2), NpcStep(left_foot_entity));
            walk_clip.add_event_to_target(
                right_foot_id,
                frame_time(19),
                NpcStep(right_foot_entity),
            );

            let run_clip = get_clip(*run_index, &graph, &mut clips);
            run_clip.add_event_to_target(left_foot_id, frame_time(1), NpcStep(left_foot_entity));
            run_clip.add_event_to_target(right_foot_id, frame_time(9), NpcStep(right_foot_entity));

            let graph_handle = AnimationGraphHandle(graphs.add(graph));
            let animations = NpcAnimations {
                graph: graph_handle.clone(),
                idle: *idle_index,
                walk: *walk_index,
                run: *run_index,
                left_foot: left_foot_id,
                right_foot: right_foot_id,
            };
            commands.insert_resource(animations);
            graph_handle
        };
        let transitions = AnimationTransitions::new();
        commands.entity(anim_player).insert((graph, transitions));
    }
}

fn get_clip<'a>(
    node: AnimationNodeIndex,
    graph: &AnimationGraph,
    clips: &'a mut Assets<AnimationClip>,
) -> &'a mut AnimationClip {
    let node = graph.get(node).unwrap();
    let clip = match &node.node_type {
        AnimationNodeType::Clip(handle) => clips.get_mut(handle),
        _ => unreachable!(),
    };
    clip.unwrap()
}

fn get_target_id(
    name: &str,
    anim_player_entity: Entity,
    children: Query<&Children>,
    targets: Query<(NameOrEntity, &AnimationTarget)>,
) -> Option<(Entity, AnimationTargetId)> {
    for child in children.iter_descendants_depth_first(anim_player_entity) {
        let Ok((child_name, target)) = targets.get(child) else {
            continue;
        };
        if child_name.to_string() == name {
            return Some((child, target.id));
        }
    }
    None
}

/// The entity is the foot. Call `.trigger()` to *get* the animation player entity.
#[derive(AnimationEvent, Reflect, Clone, Copy)]
pub(crate) struct NpcStep(pub(crate) Entity);

/// Managed by [`play_animations`]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NpcAnimationState {
    Idle,
    Walk(f32),
    Run(f32),
}

fn play_animations(
    mut query: Query<(
        &mut TnuaAnimatingState<NpcAnimationState>,
        &TnuaController,
        &AnimationPlayers,
    )>,
    mut q_animation: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<NpcAnimations>,
) {
    for (mut animating_state, controller, anim_players) in &mut query {
        let mut iter = q_animation.iter_many_mut(anim_players.iter());
        while let Some((mut anim_player, mut transitions)) = iter.fetch_next() {
            match animating_state.update_by_discriminant({
                let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
                    continue;
                };
                let speed = basis_state.running_velocity.length();
                if speed > NPC_WALK_SPEED + 0.1 {
                    NpcAnimationState::Run(speed)
                } else if speed > 0.01 {
                    NpcAnimationState::Walk(speed)
                } else {
                    NpcAnimationState::Idle
                }
            }) {
                TnuaAnimatingStateDirective::Maintain { state } => {
                    if let Some((_index, playing_animation)) =
                        anim_player.playing_animations_mut().next()
                    {
                        match state {
                            NpcAnimationState::Run(speed) => {
                                let anim_speed = speed / NPC_MAX_SPEED;
                                playing_animation.set_speed(anim_speed);
                            }
                            NpcAnimationState::Walk(speed) => {
                                let anim_speed = speed / NPC_WALK_SPEED;
                                playing_animation.set_speed(anim_speed);
                            }
                            NpcAnimationState::Idle => {}
                        }
                    }
                }
                TnuaAnimatingStateDirective::Alter {
                    // We don't need the old state here, but it's available for transition
                    // animations.
                    old_state: _,
                    state,
                } => match state {
                    NpcAnimationState::Idle => {
                        transitions
                            .play(
                                &mut anim_player,
                                animations.idle,
                                Duration::from_millis(500),
                            )
                            .repeat();
                    }
                    NpcAnimationState::Walk(_speed) => {
                        transitions
                            .play(
                                &mut anim_player,
                                animations.walk,
                                Duration::from_millis(300),
                            )
                            .repeat();
                    }
                    NpcAnimationState::Run(_speed) => {
                        transitions
                            .play(&mut anim_player, animations.run, Duration::from_millis(400))
                            .repeat();
                    }
                },
            }
        }
    }
}
