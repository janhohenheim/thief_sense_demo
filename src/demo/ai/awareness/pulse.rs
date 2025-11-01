use std::time::Duration;

use bevy::{ecs::system::SystemState, prelude::*};

use crate::{
    demo::{
        ai::{
            awareness::{AwarenessFlags, AwarenessLevel, AwarenessQuery},
            sense::SenseTimer,
        },
        npc::Npc,
        player::Player,
    },
    fixed_timer::FixedTimerApp as _,
};

pub(super) fn plugin(app: &mut App) {
    app.add_fixed_timer::<ModerateDelayTimer>();
    app.add_fixed_timer::<HighDelayTimer>();
    app.add_fixed_timer::<ModerateDelayReuseTimer>();
    app.add_fixed_timer::<HighDelayReuseTimer>();
    app.register_required_components::<Npc, ModerateDelayTimer>();
    app.register_required_components::<Npc, HighDelayTimer>();
    app.register_required_components::<Npc, ModerateDelayReuseTimer>();
    app.register_required_components::<Npc, HighDelayReuseTimer>();
}

pub(crate) struct PulseInput {
    pub(crate) npc: Entity,
    pub(crate) object: Entity,
    pub(crate) level: AwarenessLevel,
    pub(crate) is_audio: bool,
}

pub(crate) fn pulse(
    In(PulseInput {
        npc,
        object,
        level,
        is_audio,
    }): In<PulseInput>,
    world: &mut World,
    awareness_query: &mut SystemState<AwarenessQuery>,
) -> Result {
    let mut awareness = awareness_query
        .get_mut(world)
        .get(npc, object)
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
    let is_sensing_player = world.entity(object).get::<Player>().is_some();

    // alertness delay
    let level = if is_sensing_player && level > awareness.level {
        world.run_system_cached_with(
            delay_awareness,
            DelayInput {
                npc,
                old_level: awareness.level,
                new_level: level,
            },
        )?
    } else {
        level
    };

    awareness_query.get_mut(world).set(npc, object, awareness);
    Ok(())
}

struct DelayInput {
    npc: Entity,
    old_level: AwarenessLevel,
    new_level: AwarenessLevel,
}

fn delay_awareness(
    In(DelayInput {
        npc,
        old_level,
        new_level,
    }): In<DelayInput>,
    mut npcs: Query<(
        &mut ModerateDelayTimer,
        &mut HighDelayTimer,
        &mut ModerateDelayReuseTimer,
        &mut HighDelayReuseTimer,
        &mut SenseTimer,
    )>,
) -> Result<AwarenessLevel> {
    let (mut moderate_delay, mut high_delay, mut moderate_reuse, mut high_reuse, sense_timer) =
        npcs.get_mut(npc)?;

    // Early returns
    match new_level {
        AwarenessLevel::Lowest | AwarenessLevel::Low => return Ok(new_level),
        AwarenessLevel::Moderate => {
            if !moderate_delay.is_finished() {
                // The original now delays the sense timer, but that seems like a bug as it's delayed *per pulse*,
                // so if you get 5 pulses nearby, you get delayed by a factor of 5.
                // But this doesn't really affect the gameplay anyways, as it is then reset when the player is nearby anyways.
                // So that sounds like a second bug fixing the first bug lol.
                // Same is true for the `::High` branch.
                return Ok(AwarenessLevel::Low);
            } else if !moderate_reuse.is_finished() {
                return Ok(new_level);
            }
        }
        AwarenessLevel::High => {
            if !high_delay.is_finished() {
                return Ok(if moderate_delay.is_finished() {
                    AwarenessLevel::Moderate
                } else {
                    old_level
                });
            } else if !high_reuse.is_finished() {
                return Ok(new_level);
            }
        }
    }

    // Set timers
    match new_level {
        AwarenessLevel::Lowest | AwarenessLevel::Low => {
            unreachable!("This was already early returned")
        }
        AwarenessLevel::Moderate => {
            moderate_delay.reset();
            high_delay.reset();
            moderate_reuse.reset();
            high_reuse.reset();
            Ok(AwarenessLevel::Low)
        }
        AwarenessLevel::High => {
            if old_level <= AwarenessLevel::Moderate {
                moderate_delay.reset();
            }
            high_delay.reset();
            moderate_reuse.reset();
            high_reuse.reset();
            Ok(if !moderate_delay.is_finished() {
                AwarenessLevel::Low
            } else {
                AwarenessLevel::Moderate
            })
        }
    }
}

delay_timer! { struct ModerateDelayTimer(750 ms); }
delay_timer! { struct HighDelayTimer(500 ms); }
delay_timer! { struct ModerateDelayReuseTimer(12000 ms); }
delay_timer! { struct HighDelayReuseTimer(22000 ms); }

macro_rules! delay_timer {
    (struct $name:ident($duration:literal ms);) => {
        #[derive(Component, Debug, Reflect, Deref, DerefMut)]
        struct $name(Timer);

        impl Default for $name {
            fn default() -> Self {
                $name(Timer::new(
                    Duration::from_millis($duration),
                    TimerMode::Once,
                ))
            }
        }
    };
}
use delay_timer;
