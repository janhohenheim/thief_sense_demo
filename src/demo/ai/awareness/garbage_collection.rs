use bevy::prelude::*;

use crate::{
    GameFixedUpdateSystems,
    demo::ai::awareness::{
        Awareness, AwarenessLevel, AwarenessToNpc, AwarenessToObject, NpcToAwareness,
        pulse::AwarenessCapacitorDurations,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (decrease_expired_capacitors, despawn_dangling_awarenesses)
            .chain()
            .in_set(GameFixedUpdateSystems::GarbageCollection),
    );
}

fn decrease_expired_capacitors(
    mut commands: Commands,
    npcs: Query<(&AwarenessCapacitorDurations, &NpcToAwareness)>,
    mut awarenesses: Query<(Entity, &mut Awareness)>,
) {
    for (capacitor_durations, npc_to_awareness) in npcs.iter() {
        let mut awareness_iter = awarenesses.iter_many_mut(npc_to_awareness.get());
        while let Some((entity, mut awareness)) = awareness_iter.fetch_next() {
            if awareness.capacitor.is_finished() {
                awareness.level = awareness.level.decrease();
                if awareness.level == AwarenessLevel::Lowest {
                    commands.entity(entity).try_despawn();
                } else {
                    let new_cap = capacitor_durations.get(awareness.level);
                    awareness.capacitor.set_duration(new_cap);
                    awareness.capacitor.reset();
                }
            }
        }
    }
}

fn despawn_dangling_awarenesses(
    mut commands: Commands,
    awarenesses: Query<
        Entity,
        (
            With<Awareness>,
            Or<(Without<AwarenessToNpc>, Without<AwarenessToObject>)>,
        ),
    >,
) {
    for awareness in awarenesses.iter() {
        commands.entity(awareness).try_despawn();
    }
}
