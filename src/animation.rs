//! Animation system boilerplate.

use std::iter;

use bevy::{
    animation::{AnimationTarget, AnimationTargetId},
    ecs::system::QueryLens,
    prelude::*,
    scene::SceneInstanceReady,
};
pub(super) fn plugin(app: &mut App) {
    app.register_type::<AnimationPlayerOf>();
    app.register_type::<AnimationPlayers>();
    app.add_observer(link_animation_player);
}

/// Entities with this component will receive an [`AnimationPlayers`] relationship so that they can easily find the animation player of their model.
#[derive(Component)]
pub(crate) struct AnimationPlayerAncestor;

/// Simple link to the animation player of a model that is buried deep in the hierarchy.
#[derive(Component, Reflect, Clone, Deref)]
#[reflect(Component)]
#[relationship_target(relationship = AnimationPlayerOf)]
pub(crate) struct AnimationPlayers(Vec<Entity>);

#[derive(Component, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = AnimationPlayers)]
pub(crate) struct AnimationPlayerOf(pub(crate) Entity);

/// Bevy likes to hide the [`AnimationPlayer`] component deep in the hierarchy of a model.
/// This system ensures that we can find the animation player easily by inserting an [`AnimationPlayers`] relationship
/// into the same entity that contains the [`AnimationPlayerAncestor`] component.
fn link_animation_player(
    trigger: On<SceneInstanceReady>,
    mut commands: Commands,
    q_parent: Query<&ChildOf>,
    q_children: Query<&Children>,
    q_animation_player: Query<Entity, With<AnimationPlayer>>,
    q_ancestor: Query<Entity, With<AnimationPlayerAncestor>>,
) {
    let scene_root = trigger.entity;
    let animation_player = q_children
        .iter_descendants(scene_root)
        .find(|child| q_animation_player.get(*child).is_ok());
    let Some(animation_player) = animation_player else {
        return;
    };

    let animation_ancestor = iter::once(animation_player)
        .chain(q_parent.iter_ancestors(animation_player))
        .find(|entity| q_ancestor.get(*entity).is_ok());
    let Some(animation_ancestor) = animation_ancestor else {
        return;
    };

    commands
        .entity(animation_player)
        .insert(AnimationPlayerOf(animation_ancestor));
}

pub(crate) fn get_clip<'a>(
    node: AnimationNodeIndex,
    graph: &AnimationGraph,
    clips: &'a mut Assets<AnimationClip>,
) -> Result<&'a mut AnimationClip> {
    let node = graph
        .get(node)
        .ok_or_else(|| BevyError::from("Node not found"))?;
    let clip = match &node.node_type {
        AnimationNodeType::Clip(handle) => clips.get_mut(handle),
        _ => return Err("Node is not a clip".into()),
    };
    clip.ok_or_else(|| "Clip has an invalid handle".into())
}

pub(crate) fn find_bone_id(
    name: &str,
    anim_player_entity: Entity,
    children: Query<&Children>,
    targets: &mut QueryLens<(NameOrEntity, &AnimationTarget)>,
) -> Result<AnimationTargetId> {
    let targets = targets.query();
    for child in children.iter_descendants_depth_first(anim_player_entity) {
        let Ok((child_name, target)) = targets.get(child) else {
            continue;
        };
        if child_name.to_string() == name {
            return Ok(target.id);
        }
    }
    Err(BevyError::from(format!("Failed to find bone '{name}'")))
}
