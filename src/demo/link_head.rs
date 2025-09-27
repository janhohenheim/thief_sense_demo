use std::iter;

use bevy::{
    ecs::system::{IntoObserverSystem, ObserverSystem},
    prelude::*,
    scene::SceneInstanceReady,
};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

#[derive(Component, Debug)]
#[relationship(relationship_target=Head)]
pub(crate) struct HeadOf(Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship=HeadOf)]
pub(crate) struct Head(Entity);

pub(crate) fn link_head_bone<Marker: Component>(
    bone_name: impl Into<String>,
) -> impl ObserverSystem<SceneInstanceReady, ()> {
    let bone_name = bone_name.into();
    IntoObserverSystem::into_system(
        move |trigger: On<SceneInstanceReady>,
              children: Query<&Children>,
              parents: Query<&ChildOf>,
              npcs: Query<(), With<Marker>>,
              names: Query<&Name>,
              mut commands: Commands|
              -> Result {
            let Some(npc) = iter::once(trigger.entity)
                .chain(parents.iter_ancestors(trigger.entity))
                .find(|e| npcs.contains(*e))
            else {
                return Err(BevyError::from(
                    "Failed to create view cone attachment: not a descendant of an NPC",
                ));
            };
            let mut found = false;
            for child in children.iter_descendants(trigger.entity) {
                let Ok(name) = names.get(child) else {
                    continue;
                };
                if name.as_str() != bone_name.as_str() {
                    continue;
                }

                commands.entity(child).insert(HeadOf(npc));

                found = true;
                break;
            }
            if !found {
                return Err(BevyError::from(
                    "Failed to create view cone attachment: bone not found",
                ));
            }
            Ok(())
        },
    )
}
