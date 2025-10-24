//! Original implementation notes:
//! The AI iterates over the player and all close-ish NPCs. This represents the AI sensing the player and sensing other NPCs.
//! In every iteration, it checks a timer. For NPCs, it's 500 milliseconds. For the player, it's 200 milliseconds if near (about 12 meters), 500 milliseconds otherwise.
//! If the timer is due, the sensing happens. Vision is based on what the vision cones see *right now* this frame.
//! Only the highest order vision cone that contains the target is used.
//! Visibility is cached. Dunno if only the raycasts or more.
//! The sound meanwhile is buffered and considers all sounds that happened since the last time the timer was due.
//! Interestingly, all of this is only true for the AI sensing players and NPCs. Looking at suspicious objects is done completely separately, no vision cones involved.
//! Sound for e.g. thrown plates is also done separately, but I'm not sure of the timers used in both cases.

use bevy::prelude::*;

use crate::{
    AiSystems,
    demo::{
        ai::{
            awareness::AwarenessLevel, debug::DebugVision, hearing::listen::listen,
            vision::look::look,
        },
        npc::Npc,
        player::Player,
    },
    rand_timer::{RandTimer, RandTimerApp},
};

/// in seconds
pub(crate) const SENSE_INTERVAL_NEAR: f32 = 0.2;
pub(crate) const SENSE_INTERVAL_FAR: f32 = 0.5;

pub(super) fn plugin(app: &mut App) {
    app.add_rand_timer::<SenseTimer>();
    app.add_systems(
        RunFixedMainLoop,
        update_all_senses.in_set(AiSystems::Update),
    );
}

fn update_all_senses(world: &mut World) -> Result {
    let mut errors = Vec::new();
    let npcs = world.run_system_cached(get_npcs_to_update)?;
    for npc in npcs {
        world.entity_mut(npc.entity).remove::<DebugVision>();
        if let Err(err) = update_senses(In(npc), world) {
            errors.push(err);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(BevyError::from(
            errors
                .iter()
                .fold(String::new(), |acc, err| acc + &err.to_string()),
        ))
    }
}

struct ToUpdate {
    entity: Entity,
    near: bool,
}

fn get_npcs_to_update(
    player_transform: Single<&Transform, With<Player>>,
    mut npcs: Query<(Entity, &Transform, &mut SenseTimer), With<Npc>>,
) -> Vec<ToUpdate> {
    npcs.iter_mut()
        .filter_map(|(entity, npc_transform, mut timer)| {
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
            timer.set_base_time_secs_f32(secs);
            Some(ToUpdate { entity, near })
        })
        .collect::<Vec<_>>()
}

fn update_senses(In(npc): In<ToUpdate>, world: &mut World) -> Result {
    let _vision_pulses: Vec<(Entity, AwarenessLevel)> = look(In(npc.entity), world)?;
    let _hearing_pulses: Vec<(Entity, AwarenessLevel)> = listen(In((npc.entity, npc.near)), world)?;
    Ok(())
}

#[derive(Component, Debug, Deref, DerefMut)]
pub(crate) struct SenseTimer(pub(crate) RandTimer);

impl Default for SenseTimer {
    fn default() -> Self {
        Self(RandTimer::from_secs_f32(SENSE_INTERVAL_FAR))
    }
}
