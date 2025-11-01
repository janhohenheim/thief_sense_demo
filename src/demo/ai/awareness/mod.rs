use bevy::{ecs::system::SystemParam, prelude::*, time::Stopwatch};
use bitflags::bitflags;

pub(crate) mod pulse;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(pulse::plugin);
    app.add_systems(FixedPreUpdate, tick_awareness_times);
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect)]
pub(crate) enum AwarenessLevel {
    #[default]
    Lowest = 0,
    Low,
    Moderate,
    High,
}

impl AwarenessLevel {
    pub(crate) fn is_aware(self) -> bool {
        self != AwarenessLevel::Lowest
    }
}

#[derive(Component, Clone, Reflect, Debug, Default)]
#[reflect(Component)]
pub(crate) struct Awareness {
    pub(crate) level: AwarenessLevel,
    /// Time in current [`Awareness::level`]
    pub(crate) time: Stopwatch,
    pub(crate) flags: AwarenessFlags,
}

#[derive(Debug, Copy, Clone, Default, Reflect)]
pub(crate) struct AwarenessFlags(u8);
bitflags! {
    impl AwarenessFlags: u8 {
        const SEEN = 1 << 0;
        const HEARD = 1 << 1;
        const SENSED = Self::SEEN.bits() | Self::HEARD.bits();
        /// Object has an uninterrupted raycast path to it
        const CAN_RAYCAST = 1 << 2;
        /// Object has an uninterrupted raycast path to it AND it is within the NPC's view cone
        const HAS_LOS = 1 << 3;
        const FIRST_HAND = 1 << 4;
        // TODO: there's also a kAIAF_Freshened for refreshing awareness links, aka keeping them alive.
        // That is useful when an NPC needs to chase a player across the map because the player climbed a ladder.
    }
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
    awarenesses: Query<'w, 's, (&'static mut Awareness, &'static AwarenessToObject)>,
    commands: Commands<'w, 's>,
}

impl AwarenessQuery<'_, '_> {
    pub(crate) fn get(&self, npc: Entity, target: Entity) -> Option<&'_ Awareness> {
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

    pub(crate) fn set(&mut self, npc: Entity, target: Entity, awareness: Awareness) {
        let Ok(awareness_targets) = self.awareness_targets.get(npc) else {
            self.commands.spawn((
                Name::new("Awareness"),
                AwarenessToNpc(npc),
                AwarenessToObject(target),
                awareness,
            ));
            return;
        };
        let existing_awareness = awareness_targets
            // Iterating should be fine, a given NPC won't be aware of more than 3 objects or so anyways
            .iter()
            .find(|entity| {
                self.awarenesses
                    .get(*entity)
                    .is_ok_and(|(_awareness, awareness_to_object)| awareness_to_object.0 == target)
            });
        let Some(existing_awareness) = existing_awareness else {
            self.commands.spawn((
                Name::new("Awareness"),
                AwarenessToNpc(npc),
                AwarenessToObject(target),
                awareness,
            ));
            return;
        };
        *self.awarenesses.get_mut(existing_awareness).unwrap().0 = awareness;
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
