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
    demo::{
        ai::{awareness::AwarenessLevel, hearing::listen::listen, vision::look::look},
        npc::Npc,
    },
    rand_timer::{RandTimer, RandTimerApp},
};

pub(super) fn plugin(app: &mut App) {
    app.add_rand_timer::<SenseTimer>();
    app.add_systems(
        RunFixedMainLoop,
        update_all_senses.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
    );
}

fn update_all_senses(world: &mut World) -> Result {
    let npcs = world
        .query_filtered::<Entity, With<Npc>>()
        .iter(world)
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    for npc in npcs {
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

fn update_senses(In(npc): In<Entity>, world: &mut World) -> Result {
    let _vision_pulses: Vec<(Entity, AwarenessLevel)> = look(In(npc), world)?;
    let _hearing_pulses: Vec<(Entity, AwarenessLevel)> = listen(In(npc), world)?;
    Ok(())
}

#[derive(Component, Debug, Deref, DerefMut)]
pub(crate) struct SenseTimer(pub(crate) RandTimer);

impl Default for SenseTimer {
    fn default() -> Self {
        Self(RandTimer::from_millis(500))
    }
}
