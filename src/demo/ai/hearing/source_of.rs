use avian3d::prelude::ColliderOf;
use bevy::{
    ecs::{
        error::CommandWithEntity as _,
        lifecycle::HookContext,
        relationship::{Relationship, RelationshipHookMode, RelationshipSourceCollection},
        world::DeferredWorld,
    },
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[component(immutable, on_insert = <AiSourceOf as Relationship>::on_insert, on_replace = <AiSourceOf as Relationship>::on_replace)]
#[reflect(Component)]
pub(crate) struct AiSourceOf {
    pub(crate) body: Entity,
}

/// Intentionally not `linked_spawn` in order to allow sources to linger post-despawn
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = AiSourceOf)]
pub(crate) struct AiSources(Vec<Entity>);

// Bevy does not currently allow relationships that point to their own entity,
// so we implement the relationship manually to work around this limitation.
impl Relationship for AiSourceOf {
    type RelationshipTarget = AiSources;

    fn get(&self) -> Entity {
        self.body
    }

    fn from(entity: Entity) -> Self {
        Self { body: entity }
    }

    fn on_insert(
        mut world: DeferredWorld,
        HookContext {
            entity,
            caller,
            relationship_hook_mode,
            ..
        }: HookContext,
    ) {
        // This is largely the same as the default implementation,
        // but does not panic if the relationship target does not exist.
        //
        match relationship_hook_mode {
            RelationshipHookMode::Run => {}
            RelationshipHookMode::Skip => return,
            RelationshipHookMode::RunIfNotLinked => {
                if <Self::RelationshipTarget as RelationshipTarget>::LINKED_SPAWN {
                    return;
                }
            }
        }
        let target_entity = world.entity(entity).get::<Self>().unwrap().get();

        // For one-to-one relationships, remove existing relationship before adding new one
        let current_source_to_remove = world
            .get_entity(target_entity)
            .ok()
            .and_then(|target_entity_ref| target_entity_ref.get::<Self::RelationshipTarget>())
            .and_then(|relationship_target| {
                relationship_target
                    .collection()
                    .source_to_remove_before_add()
            });

        if let Some(current_source) = current_source_to_remove {
            world.commands().entity(current_source).try_remove::<Self>();
        }

        if let Ok(mut entity_commands) = world.commands().get_entity(target_entity) {
            // Deferring is necessary for batch mode
            entity_commands
                .entry::<Self::RelationshipTarget>()
                .and_modify(move |mut relationship_target| {
                    relationship_target.collection_mut_risky().add(entity);
                })
                .or_insert_with(move || {
                    let mut target = Self::RelationshipTarget::with_capacity(1);
                    target.collection_mut_risky().add(entity);
                    target
                });
        } else {
            warn!(
                "{}The {}({target_entity:?}) relationship on entity {entity:?} relates to an entity that does not exist. The invalid {} relationship has been removed.",
                caller
                    .map(|location| format!("{location}: "))
                    .unwrap_or_default(),
                DebugName::type_name::<Self>(),
                DebugName::type_name::<Self>()
            );
            world.commands().entity(entity).remove::<Self>();
        }
    }

    fn set_risky(&mut self, entity: Entity) {
        self.body = entity;
    }
}
