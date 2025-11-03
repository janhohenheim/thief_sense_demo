//! Original implementation notes:
//! The AI iterates over the player and all close-ish NPCs. This represents the AI sensing the player and sensing other NPCs.
//! In every iteration, it checks a timer. For NPCs, it's 500 milliseconds. For the player, it's 200 milliseconds if near (about 12 meters), 500 milliseconds otherwise.
//! If the timer is due, the sensing happens. Vision is based on what the vision cones see *right now* this frame.
//! Only the highest order vision cone that contains the target is used.
//! Visibility is cached. Dunno if only the raycasts or more.
//! The sound meanwhile is buffered and considers all sounds that happened since the last time the timer was due.
//! Interestingly, all of this is only true for the AI sensing players and NPCs. Looking at suspicious objects is done completely separately, no vision cones involved.
//! Sound for e.g. thrown plates is also done separately, but I'm not sure of the timers used in both cases.

use std::time::Duration;

use bevy::prelude::*;

use crate::{
    GameFixedUpdateSystems,
    demo::{
        ai::{
            awareness::{
                AwarenessLevel,
                pulse::{PulseInput, pulse},
            },
            debug::DebugVision,
            hearing::listen::listen,
            vision::look::look,
        },
        npc::Npc,
        player::Player,
    },
    staggered_timer::{StaggeredTimer, StaggeredTimerApp as _},
};

/// in seconds. Original uses 0.2 and 0.5, but having exact multiples allows us to just simulate the short audio stuff n times in a long frame.
/// These numbers are also neat because the sampling rate (8k Hz) times the interval is an integer.
pub(crate) const SENSE_INTERVAL_NEAR: f32 = 0.175;
pub(crate) const SENSE_INTERVAL_NEAR_TO_FAR: usize = 3;
pub(crate) const SENSE_INTERVAL_FAR: f32 = SENSE_INTERVAL_NEAR * SENSE_INTERVAL_NEAR_TO_FAR as f32;

pub(super) fn plugin(app: &mut App) {
    app.add_staggered_timer::<SenseTimer>();
    app.add_systems(
        FixedUpdate,
        update_all_senses.in_set(GameFixedUpdateSystems::Senses),
    );
    app.register_required_components::<Npc, SenseTimer>();
}

fn update_all_senses(world: &mut World, mut buff_local: Local<Option<Vec<ToUpdate>>>) -> Result {
    let mut buff = buff_local.take().unwrap_or_default();
    buff.clear();
    let npcs = world.run_system_cached_with(get_npcs_to_update, buff)?;
    for npc in &npcs {
        world.entity_mut(npc.entity).remove::<DebugVision>();
        if let Err(err) = update_senses(In(*npc), world) {
            error!("{err}");
        }
    }
    buff_local.replace(npcs);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct ToUpdate {
    entity: Entity,
    near: bool,
}

fn get_npcs_to_update(
    In(mut buff): In<Vec<ToUpdate>>,
    player_transform: Single<&Transform, With<Player>>,
    mut npcs: Query<(Entity, &Transform, &mut SenseTimer), With<Npc>>,
    time: Res<Time>,
) -> Vec<ToUpdate> {
    let to_update = npcs
        .iter_mut()
        .filter_map(|(entity, npc_transform, mut timer)| {
            timer.tick(time.delta());
            if !timer.is_finished() {
                return None;
            }
            const DIST_CUTOFF: f32 = 12.0;
            let (near, secs) = if player_transform
                .translation
                .distance_squared(npc_transform.translation)
                > DIST_CUTOFF * DIST_CUTOFF
            {
                (false, SENSE_INTERVAL_FAR)
            } else {
                (true, SENSE_INTERVAL_NEAR)
            };
            timer.reset_with(Duration::from_secs_f32(secs));
            Some(ToUpdate { entity, near })
        });
    buff.extend(to_update);
    buff
}

fn update_senses(In(npc): In<ToUpdate>, world: &mut World) -> Result {
    let vision_pulses: Vec<(Entity, AwarenessLevel)> = look(In(npc.entity), world)?;
    for (vision_entity, vision_level) in vision_pulses {
        match world.run_system_cached_with(
            pulse,
            PulseInput {
                npc: npc.entity,
                object: vision_entity,
                level: vision_level,
                is_audio: false,
            },
        ) {
            Ok(()) => (),
            Err(err) => error!("Error updating vision sense: {}", err),
        }
    }
    let hearing_pulses: Vec<(Entity, AwarenessLevel)> =
        world.run_system_cached_with(listen, (npc.entity, npc.near))?;
    for (hearing_entity, hearing_level) in hearing_pulses {
        match world.run_system_cached_with(
            pulse,
            PulseInput {
                npc: npc.entity,
                object: hearing_entity,
                level: hearing_level,
                is_audio: true,
            },
        ) {
            Ok(()) => (),
            Err(err) => error!("Error updating hearing sense: {}", err),
        }
    }
    Ok(())
}

#[derive(Component, Debug, Deref, DerefMut)]
pub(crate) struct SenseTimer(pub(crate) StaggeredTimer);

impl Default for SenseTimer {
    fn default() -> Self {
        Self(StaggeredTimer::new(Duration::from_secs_f32(
            SENSE_INTERVAL_FAR,
        )))
    }
}
