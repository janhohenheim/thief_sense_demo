use bevy::prelude::*;

use crate::{
    GameFixedUpdateSystems,
    demo::{
        ai::{alertness::update_alertness, sense::update_senses},
        npc::Npc,
    },
};

pub(crate) mod alertness;
pub(crate) mod awareness;

pub(crate) mod debug;
pub(crate) mod hearing;
pub(crate) mod sense;
pub(crate) mod vision;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        sense::plugin,
        awareness::plugin,
        vision::plugin,
        hearing::plugin,
        alertness::plugin,
        debug::plugin,
    ));
    app.add_systems(
        FixedUpdate,
        update_ai.in_set(GameFixedUpdateSystems::AiSenses),
    );
}

fn update_ai(world: &mut World, mut buff_local: Local<Option<Vec<Entity>>>) -> Result {
    let mut npcs = buff_local.take().unwrap_or_default();
    npcs.extend(world.query_filtered::<Entity, With<Npc>>().iter(world));
    for npc in npcs.drain(..) {
        () = world.run_system_cached_with(update_senses, npc)?;
        () = world.run_system_cached_with(update_alertness, npc)?;
    }
    buff_local.replace(npcs);
    Ok(())
}
