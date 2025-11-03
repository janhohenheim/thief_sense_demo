use std::time::Duration;

use bevy::{ecs::system::SystemState, prelude::*};

use crate::{
    demo::{
        ai::{
            alertness::Alertness,
            awareness::{AwarenessFlags, AwarenessLevel, AwarenessQuery},
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
    app.register_required_components::<Npc, AwarenessCapacitorDurations>();
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
        level: pulse,
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
    if pulse.is_aware() {
        if is_audio {
            awareness.flags.insert(AwarenessFlags::HEARD);
        } else {
            awareness.flags.insert(AwarenessFlags::SEEN);
        }
    }
    let is_sensing_player = world.entity(object).get::<Player>().is_some();

    // alertness delay
    let pulse = if is_sensing_player && pulse > awareness.level {
        world.run_system_cached_with(
            delay_awareness,
            DelayInput {
                npc,
                old_level: awareness.level,
                new_level: pulse,
            },
        )?
    } else {
        pulse
    };

    let alertness = *world
        .entity(npc)
        .get::<Alertness>()
        .ok_or("Alertness component not found")?;

    if awareness.flags.intersects(AwarenessFlags::SENSED)
        || awareness.last_true_contact.elapsed() < alertness.free_knowledge
    {
        if awareness.flags.intersects(AwarenessFlags::SENSED) {
            awareness.last_true_contact.reset();
            awareness.last_contact.reset();
        }
        // I think the original here does two more sophisticated things:
        // - Use the last probe passed by the sound (idk if Steam Audio allows us to retrieve that info)
        // - Sample the best point on the "navmesh". Not quite navmesh because it instead looks for the best cell in the world and then works with that.
        let object_pos = world
            .entity(object)
            .get::<GlobalTransform>()
            .ok_or("GlobalTransform component not found")?
            .translation();

        awareness.last_pos = object_pos;
    }
    awareness.last_pulse = pulse;

    let capacitor_durations = *world
        .entity(npc)
        .get::<AwarenessCapacitorDurations>()
        .ok_or("Capacitor component not found")?;
    // If the AI is aware of the object and is currently sensing it,
    // ensure we remain aware of it.
    if pulse.is_aware() && awareness.level == AwarenessLevel::High {
        awareness
            .capacitor
            .set_duration(capacitor_durations.get(awareness.level));
        awareness.capacitor.reset();
    } else if pulse >= awareness.level {
        awareness.level = pulse;
        awareness
            .capacitor
            .set_duration(capacitor_durations.get(awareness.level));
        awareness.capacitor.reset();
    }
    // The branch for the capacitor expiring is handled in the garbage collection.
    awareness_query.get_mut(world).set(npc, object, awareness);
    awareness_query.apply(world);
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
    )>,
) -> Result<AwarenessLevel> {
    let (mut moderate_delay, mut high_delay, mut moderate_reuse, mut high_reuse) =
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
        #[reflect(Component)]
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

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct AwarenessCapacitorDurations {
    low_to_lowest: Duration,
    moderate_to_low: Duration,
    high_to_moderate: Duration,
}

impl Default for AwarenessCapacitorDurations {
    fn default() -> Self {
        Self {
            low_to_lowest: Duration::from_millis(4000),
            moderate_to_low: Duration::from_millis(8000),
            high_to_moderate: Duration::from_millis(45000),
        }
    }
}

impl AwarenessCapacitorDurations {
    pub(crate) fn get(self, awareness: AwarenessLevel) -> Duration {
        match awareness {
            AwarenessLevel::Lowest => Duration::MAX,
            AwarenessLevel::Low => self.low_to_lowest,
            AwarenessLevel::Moderate => self.moderate_to_low,
            AwarenessLevel::High => self.high_to_moderate,
        }
    }
}
