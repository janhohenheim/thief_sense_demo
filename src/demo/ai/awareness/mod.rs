use avian3d::prelude::SpatialQuery;
use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*, time::Stopwatch};
use evergreen_relations::prelude::*;

use crate::demo::{ai::sense::SenseTimer, npc::Npc};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(FixedPreUpdate, tick_awareness_times);
    app.register_required_components::<Npc, Alertness>();
}

#[derive(Debug, Component, Default)]
pub(crate) struct Alertness(pub(crate) AwarenessLevel);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect)]
pub(crate) enum AwarenessLevel {
    #[default]
    Lowest = 0,
    Low,
    Moderate,
    High,
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub(crate) struct Awareness {
    pub(crate) level: AwarenessLevel,
    /// Time in current [`Awareness::level`]
    pub(crate) time: Stopwatch,
}

impl Awareness {
    #[expect(dead_code)]
    pub(crate) fn set_level(&mut self, level: AwarenessLevel) {
        if self.level != level {
            self.level = level;
            self.time.reset();
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct AwarenessQuery<'w, 's> {
    awareness_targets: Query<'w, 's, &'static AwarenessToNpcTarget>,
    awarenesses: Query<'w, 's, (&'static Awareness, &'static AwarenessToObject)>,
}

impl AwarenessQuery<'_, '_> {
    pub(crate) fn get_awareness_of(&self, npc: Entity, target: Entity) -> Option<&'_ Awareness> {
        let awareness_targets = self.awareness_targets.get(npc).ok()?;
        awareness_targets
            // Iterating should be fine, a given NPC won't be aware of more than 3 objects or so anyways
            .iter()
            .filter_map(|awareness| self.awarenesses.get(awareness).ok())
            .find_map(|(awareness, awareness_to_object)| {
                if awareness_to_object.0 == target {
                    Some(awareness)
                } else {
                    None
                }
            })
    }
}

#[derive(Component)]
#[relationship(relationship_target = AwarenessToObjectTarget)]
pub(crate) struct AwarenessToObject(Entity);

#[derive(Component)]
#[relationship_target(relationship = AwarenessToObject)]
pub(crate) struct AwarenessToObjectTarget(Entity);

#[derive(Component)]
#[relationship(relationship_target = AwarenessToNpcTarget)]
pub(crate) struct AwarenessToNpc(Entity);

#[derive(Component)]
#[relationship_target(relationship = AwarenessToNpc, linked_spawn)]
pub(crate) struct AwarenessToNpcTarget(Vec<Entity>);

fn tick_awareness_times(mut awareness: Query<&mut Awareness>, time: Res<Time>) {
    for mut awareness in awareness.iter_mut() {
        awareness.time.tick(time.delta());
    }
}

pub(crate) struct SetAwareCommand {
    pub(crate) target: Entity,
    pub(crate) awareness: Awareness,
}

impl EntityCommand for SetAwareCommand {
    fn apply(self, entity_world: EntityWorldMut) {
        let entity = entity_world.id();
        entity_world.into_world_mut().spawn((
            Name::new("Awareness"),
            AwarenessToNpc(entity),
            AwarenessToObject(self.target),
            self.awareness,
        ));
    }
}

pub(crate) trait SetAwareCommandExt {
    fn set_awareness_of(&mut self, target: Entity, awareness: Awareness) -> &mut Self;
}

impl SetAwareCommandExt for EntityCommands<'_> {
    fn set_awareness_of(&mut self, target: Entity, awareness: Awareness) -> &mut Self {
        self.queue(SetAwareCommand { target, awareness })
    }
}
