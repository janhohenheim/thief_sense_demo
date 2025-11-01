use std::time::Duration;

use bevy::{
    ecs::{system::SystemState, world::DeferredWorld},
    prelude::*,
};

use crate::{
    demo::{
        ai::awareness::{AwarenessFlags, AwarenessLevel, AwarenessQuery},
        npc::Npc,
        player::Player,
    },
    fixed_timer::FixedTimerApp as _,
};

pub(super) fn plugin(app: &mut App) {
    app.register_required_components::<Npc, Alertness>();
    app.add_fixed_timer::<ModerateDelayTimer>();
}

#[derive(Debug, Component, Default)]
#[require(
    ModerateDelayTimer,
    HighDelayTimer,
    ModerateDelayReuseTimer,
    HighDelayReuseTimer
)]
pub(crate) struct Alertness(pub(crate) AwarenessLevel);

pub(crate) struct PulseInput {
    pub(crate) entity: Entity,
    pub(crate) object: Entity,
    pub(crate) level: AwarenessLevel,
    pub(crate) is_audio: bool,
}

pub(crate) fn pulse(
    In(PulseInput {
        entity,
        object,
        level,
        is_audio,
    }): In<PulseInput>,
    world: &mut World,
    awareness_query: &mut SystemState<AwarenessQuery>,
) -> Result {
    let mut awareness = awareness_query
        .get_mut(world)
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
    let is_sensing_player = world.entity(object).get::<Player>().is_some();

    // alertness delay
    if is_sensing_player && level > awareness.level {
        world.run_system_cached_with(
            delay_awareness,
            PulseInput {
                entity,
                object,
                level,
                is_audio,
            },
        )?;
    }

    awareness_query
        .get_mut(world)
        .set(entity, object, awareness);
    Ok(())
}

fn delay_awareness(
    In(PulseInput {
        entity,
        object,
        level,
        is_audio,
    }): In<PulseInput>,
) {
    match level {
        AwarenessLevel::Lowest | AwarenessLevel::Low => { /* No delay */ }
        AwarenessLevel::Moderate => todo!(),
        AwarenessLevel::High => todo!(),
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
