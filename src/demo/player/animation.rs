//! Player animation handling.

use std::time::Duration;

use bevy::prelude::*;
use bevy_tnua::{TnuaAnimatingState, TnuaAnimatingStateDirective, prelude::*};

use crate::{
    AppSystems,
    animation::AnimationPlayers,
    demo::player::{PLAYER_GLTF, PLAYER_RUN_SPEED, PLAYER_WALK_SPEED},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PlayerAnimations>();
    app.add_systems(
        Update,
        play_animations
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );
    app.add_observer(setup_player_animations);
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct PlayerAnimations {
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    run: AnimationNodeIndex,
}

pub(crate) fn setup_player_animations(
    trigger: On<Add, AnimationPlayers>,
    q_anim_players: Query<&AnimationPlayers>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    gltfs: Res<Assets<Gltf>>,
) {
    let gltf = gltfs.get(assets.load(PLAYER_GLTF).id()).unwrap();
    let anim_players = q_anim_players.get(trigger.entity).unwrap();
    for anim_player in anim_players.iter() {
        let (graph, indices) = AnimationGraph::from_clips(
            ["Idle_Loop", "Walk_Loop", "Sprint_Loop"]
                .map(|name| gltf.named_animations[name].clone()),
        );
        let [idle_index, walk_index, run_index] = indices.as_slice() else {
            panic!("Failed to map animation indices")
        };
        let graph_handle = graphs.add(graph);

        let animations = PlayerAnimations {
            idle: *idle_index,
            walk: *walk_index,
            run: *run_index,
        };
        let transitions = AnimationTransitions::new();
        commands.entity(anim_player).insert((
            animations,
            AnimationGraphHandle(graph_handle),
            transitions,
        ));
    }
}

/// Managed by [`play_animations`]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PlayerAnimationState {
    Idle,
    Walk(f32),
    Run(f32),
}

fn play_animations(
    mut query: Query<(
        &mut TnuaAnimatingState<PlayerAnimationState>,
        &TnuaController,
        &AnimationPlayers,
    )>,
    mut q_animation: Query<(
        &PlayerAnimations,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
) {
    for (mut animating_state, controller, anim_players) in &mut query {
        let mut iter = q_animation.iter_many_mut(anim_players.iter());
        while let Some((animations, mut anim_player, mut transitions)) = iter.fetch_next() {
            match animating_state.update_by_discriminant({
                let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
                    continue;
                };
                let speed = basis_state.running_velocity.length();
                if speed > PLAYER_WALK_SPEED + 0.1 {
                    PlayerAnimationState::Run(speed)
                } else if speed > 0.01 {
                    PlayerAnimationState::Walk(speed)
                } else {
                    PlayerAnimationState::Idle
                }
            }) {
                TnuaAnimatingStateDirective::Maintain { state } => {
                    if let Some((_index, playing_animation)) =
                        anim_player.playing_animations_mut().next()
                    {
                        match state {
                            PlayerAnimationState::Run(speed) => {
                                let anim_speed = speed / PLAYER_RUN_SPEED;
                                playing_animation.set_speed(anim_speed);
                            }
                            PlayerAnimationState::Walk(speed) => {
                                let anim_speed = speed / PLAYER_WALK_SPEED;
                                playing_animation.set_speed(anim_speed);
                            }
                            PlayerAnimationState::Idle => {}
                        }
                    }
                }
                TnuaAnimatingStateDirective::Alter {
                    // We don't need the old state here, but it's available for transition
                    // animations.
                    old_state: _,
                    state,
                } => match state {
                    PlayerAnimationState::Idle => {
                        transitions
                            .play(
                                &mut anim_player,
                                animations.idle,
                                Duration::from_millis(500),
                            )
                            .repeat();
                    }
                    PlayerAnimationState::Walk(_speed) => {
                        transitions
                            .play(
                                &mut anim_player,
                                animations.walk,
                                Duration::from_millis(300),
                            )
                            .repeat();
                    }
                    PlayerAnimationState::Run(_speed) => {
                        transitions
                            .play(&mut anim_player, animations.run, Duration::from_millis(400))
                            .repeat();
                    }
                },
            }
        }
    }
}
