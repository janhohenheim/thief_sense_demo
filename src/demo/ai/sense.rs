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
    app.register_required_components::<Npc, SenseTimer>();
}

pub(crate) fn update_senses(In(npc): In<Entity>, world: &mut World) -> Result {
    let should_update: Option<ShouldUpdate> =
        world.run_system_cached_with(should_update_npc, npc)?;
    let Some(ShouldUpdate { near }) = should_update else {
        return Ok(());
    };
    world.entity_mut(npc).remove::<DebugVision>();
    let vision_pulses: Vec<(Entity, AwarenessLevel)> = look(In(npc), world)?;
    for (vision_entity, vision_level) in vision_pulses {
        match world.run_system_cached_with(
            pulse,
            PulseInput {
                npc,
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
        world.run_system_cached_with(listen, (npc, near))?;
    for (hearing_entity, hearing_level) in hearing_pulses {
        match world.run_system_cached_with(
            pulse,
            PulseInput {
                npc,
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

struct ShouldUpdate {
    near: bool,
}

fn should_update_npc(
    In(npc): In<Entity>,
    player_transform: Single<&GlobalTransform, With<Player>>,
    mut npcs: Query<(&GlobalTransform, &mut SenseTimer), With<Npc>>,
    time: Res<Time>,
) -> Result<Option<ShouldUpdate>> {
    let (npc_transform, mut sense_timer) = npcs.get_mut(npc)?;
    sense_timer.tick(time.delta());
    if !sense_timer.is_finished() {
        return Ok(None);
    }
    const DIST_CUTOFF: f32 = 12.0;
    let (near, secs) = if player_transform
        .translation()
        .distance_squared(npc_transform.translation())
        > DIST_CUTOFF * DIST_CUTOFF
    {
        (false, SENSE_INTERVAL_FAR)
    } else {
        (true, SENSE_INTERVAL_NEAR)
    };
    sense_timer.reset_with(Duration::from_secs_f32(secs));
    Ok(Some(ShouldUpdate { near }))
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

/// f32[0, 1] -> u8[0, 100]
pub(crate) fn calc_control_rating(fraction: f32, low: u8, mid: u8, high: u8) -> u8 {
    let raw = (fraction * 100.0).clamp(1.0, 100.0) as u8;
    const LOW_NORM: u8 = 25;
    const MID_NORM: u8 = 50;
    const HIGH_NORM: u8 = 75;
    let (pre_norm_base, pre_norm_range, norm_base, norm_range) = match raw {
        l if l < low => (0, low, 0, LOW_NORM),
        l if l < mid => (low, mid - low, LOW_NORM, MID_NORM - LOW_NORM),
        l if l < high => (mid, high - mid, MID_NORM, HIGH_NORM - MID_NORM),
        _ => (high, 100 - high, HIGH_NORM, 100 - HIGH_NORM),
    };

    norm_base + ((raw - pre_norm_base) as f32 / pre_norm_range as f32) as u8 + norm_range
}
