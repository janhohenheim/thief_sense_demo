use bevy::{ecs::system::SystemParam, prelude::*, time::Stopwatch};
use bitflags::bitflags;
use strum::EnumCount;

pub(crate) mod free_knowledge;
pub(crate) mod pulse;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((pulse::plugin, free_knowledge::plugin));
    app.add_systems(FixedPreUpdate, (tick_awareness_times, count_awareness));
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Reflect, EnumCount)]
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
    pub(crate) fn decrease(self) -> Self {
        match self {
            AwarenessLevel::Lowest => AwarenessLevel::Lowest,
            AwarenessLevel::Low => AwarenessLevel::Lowest,
            AwarenessLevel::Moderate => AwarenessLevel::Low,
            AwarenessLevel::High => AwarenessLevel::Moderate,
        }
    }
}

#[derive(Component, Clone, Reflect, Debug, Default)]
#[reflect(Component)]
pub(crate) struct Awareness {
    /// Current [`AwarenessLevel`]
    pub(crate) level: AwarenessLevel,
    /// Time left in current [`Awareness::level`].
    /// If the awareness level is lower than the previous level when this timer is expired, the awareness level is decreased.
    pub(crate) capacitor: Timer,
    /// Flags of the current awareness
    pub(crate) flags: AwarenessFlags,
    /// Last time there was a sensation, i.e. the NPC either heard or saw something
    pub(crate) last_true_contact: Stopwatch,
    /// Last position of the sensed object.
    pub(crate) last_pos: Vec3,
    /// Last pulse contributing to the awareness
    pub(crate) last_pulse: AwarenessLevel,
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

#[derive(SystemParam)]
pub(crate) struct AwarenessQuery<'w, 's> {
    npc_to_awareness: Query<'w, 's, &'static NpcToAwareness>,
    awarenesses: Query<'w, 's, (&'static mut Awareness, &'static AwarenessToObject)>,
    commands: Commands<'w, 's>,
}

impl AwarenessQuery<'_, '_> {
    pub(crate) fn get(&self, npc: Entity, object: Entity) -> Option<&'_ Awareness> {
        let npc_to_awareness = self.npc_to_awareness.get(npc).ok()?;
        let res = npc_to_awareness
            // Iterating should be fine, a given NPC won't be aware of more than 3 objects or so anyways
            .iter()
            .filter_map(|awareness| self.awarenesses.get(awareness).ok())
            .find_map(|(awareness, awareness_to_object)| {
                if awareness_to_object.0 == object {
                    Some(awareness)
                } else {
                    None
                }
            });
        info!("res: {}", res.is_some());
        res
    }

    pub(crate) fn set(&mut self, npc: Entity, object: Entity, awareness: Awareness) {
        let Ok(awareness_targets) = self.npc_to_awareness.get(npc) else {
            info!("spawned");
            self.commands.spawn((
                Name::new("Awareness"),
                AwarenessToNpc(npc),
                AwarenessToObject(object),
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
                    .is_ok_and(|(_awareness, awareness_to_object)| awareness_to_object.0 == object)
            });
        let Some(existing_awareness) = existing_awareness else {
            self.commands.spawn((
                Name::new("Awareness"),
                AwarenessToNpc(npc),
                AwarenessToObject(object),
                awareness,
            ));
            return;
        };
        *self.awarenesses.get_mut(existing_awareness).unwrap().0 = awareness;
    }
}

fn count_awareness(q: Query<(), With<Awareness>>) {
    info!("Counting awareness: {}", q.count());
}

#[derive(Component)]
#[relationship(relationship_target = ObjectToAwareness)]
pub(crate) struct AwarenessToObject(Entity);

#[derive(Component)]
#[relationship_target(relationship = AwarenessToObject, linked_spawn)]
pub(crate) struct ObjectToAwareness(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = NpcToAwareness)]
pub(crate) struct AwarenessToNpc(Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = AwarenessToNpc, linked_spawn)]
pub(crate) struct NpcToAwareness(Vec<Entity>);

fn tick_awareness_times(mut awareness: Query<&mut Awareness>, time: Res<Time>) {
    for mut awareness in awareness.iter_mut() {
        awareness.capacitor.tick(time.delta());
        awareness.last_true_contact.tick(time.delta());
    }
}
