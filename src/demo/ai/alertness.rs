use bevy::{ecs::world::DeferredWorld, prelude::*};

use crate::demo::{
    ai::awareness::{AwarenessFlags, AwarenessLevel, AwarenessQuery},
    npc::Npc,
    player::Player,
};

pub(super) fn plugin(app: &mut App) {
    app.register_required_components::<Npc, Alertness>();
}

#[derive(Debug, Component, Default)]
pub(crate) struct Alertness(pub(crate) AwarenessLevel);

pub(crate) struct PulseInput {
    entity: Entity,
    object: Entity,
    level: AwarenessLevel,
    is_audio: bool,
}

fn pulse(
    In(PulseInput {
        entity,
        object,
        level,
        is_audio,
    }): In<PulseInput>,
    mut npcs: Query<&mut Alertness>,
    player: Query<(), With<Player>>,
    mut awareness_query: AwarenessQuery,
) -> Result {
    let mut alertness = npcs.get_mut(entity)?;
    let mut awareness = awareness_query
        .get(entity, object)
        .cloned()
        .unwrap_or_default();
    awareness.flags.remove(AwarenessFlags::SENSED);
    if level.is_aware() {
        if is_audio {
            awareness.flags.insert(AwarenessFlags::HEARD);
        } else {
            awareness.flags.insert(AwarenessFlags::SEEN);
        }
    }
    awareness_query.set(entity, object, awareness);
    if player.contains(object) && level > alertness.0 && level >= AwarenessLevel::Moderate {
        match level {}
    }
    Ok(())
}
