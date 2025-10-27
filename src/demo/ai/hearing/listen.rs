use bevy::prelude::*;

use crate::demo::ai::{
    awareness::AwarenessLevel,
    calc_control_rating,
    debug::DebugHearing,
    hearing::{
        AiLoudnessControl, AiSource, LoudnessAcuity,
        accumulator::AudioInputs,
        loudness::{LoudnessInput, simulate_raw_loudness_to_listener},
        simulate::{AiSimulationInputs, update_simulation_for_listener},
    },
};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

pub(crate) fn listen(
    In((npc, near)): In<(Entity, bool)>,
    world: &mut World,
    mut buff_local: Local<Option<Vec<Entity>>>,
) -> Result<Vec<(Entity, AwarenessLevel)>> {
    let buff = buff_local.take().unwrap_or_default();
    let sources: Vec<_> = world.run_system_cached_with(sources_for_listener, (npc, buff))?;

    () = world.run_system_cached_with(
        update_simulation_for_listener,
        AiSimulationInputs {
            listener: npc,
            sources: sources.clone(),
        },
    )?;
    let mut pulses = Vec::new();
    let mut avg_loudness = 0.0;
    let source_len = sources.len();
    for source in sources.iter().copied() {
        let raw_loudness: f32 = world.run_system_cached_with(
            simulate_raw_loudness_to_listener,
            LoudnessInput {
                listener: npc,
                source,
                near,
            },
        )?;
        avg_loudness += loudness_to_fraction(raw_loudness) as f32;
        let loudness: u8 = match world.run_system_cached_with(
            loudness_to_listener,
            LoudnessToListenerInput {
                listener: npc,
                source,
                rms: raw_loudness,
            },
        ) {
            Ok(loudness) => loudness,
            Err(err) => {
                error!("Error calculating loudness: {err}");
                continue;
            }
        };

        let pulse = match loudness {
            v if v < 25 => AwarenessLevel::Lowest,
            v if v < 50 => AwarenessLevel::Low,
            v if v < 75 => AwarenessLevel::Moderate,
            _ => AwarenessLevel::High,
        };
        pulses.push((source, pulse));
    }
    if source_len != 0 {
        avg_loudness /= source_len as f32;
    }
    world.entity_mut(npc).insert(DebugHearing(avg_loudness));

    buff_local.replace(sources);

    Ok(pulses)
}

fn sources_for_listener(
    In((npc, mut buff)): In<(Entity, Vec<Entity>)>,
    transform: Query<&GlobalTransform>,
    sources: Query<(Entity, &GlobalTransform, AudioInputs), With<AiSource>>,
) -> Result<Vec<Entity>> {
    let npc_translation = transform.get(npc)?.translation();
    let sources = sources.iter().filter_map(|(entity, transform, inputs)| {
        let inputs = inputs.get().ok()?;
        let dist_squared = transform
            .translation()
            .distance_squared(npc_translation)
            .max(1.0);
        let loudness_at_dist = inputs.loudness / dist_squared;
        let fraction = loudness_to_fraction(loudness_at_dist);

        if fraction > 0.01 { Some(entity) } else { None }
    });
    buff.extend(sources);
    Ok(buff)
}

struct LoudnessToListenerInput {
    listener: Entity,
    source: Entity,
    rms: f32,
}

fn loudness_to_listener(
    In(LoudnessToListenerInput {
        listener,
        source,
        rms,
    }): In<LoudnessToListenerInput>,
    loudness_control: Query<&AiLoudnessControl>,
    acuity: Query<&LoudnessAcuity>,
) -> Result<u8> {
    let control = loudness_control.get(source)?;
    let acuity = acuity.get(listener)?;
    let fraction = loudness_to_fraction(rms);
    let rating = calc_control_rating(
        fraction,
        control.low_loudness,
        control.medium_loudness,
        control.high_loudness,
    );
    let result = (rating as f32 * acuity.0).clamp(0.0, 100.0) as u8;
    Ok(result)
}

#[inline]
fn loudness_to_fraction(rms: f32) -> f32 {
    // Tweak P0 such that the sound of footsteps at a distance of 4 meters is between 35-40 dB
    // Source: <https://www.scirp.org/journal/paperinformation?paperid=98579>
    const P0: f32 = 90e-6;
    let db_spl = 20.0 * (rms / P0).log10();
    // Human hearing threshold is around 0 dB
    // We could use 30 dB, which is a very quiet room: <http://www.makeitlouder.com/Decibel%20Level%20Chart.txt>
    // But that would make it impossible for the AI to hear teeeny tiny steps the player is taking, which is kinda cool.
    const MIN_DB: f32 = 0.0;
    // 70 dB: Safe maximum longterm exposure without hearing loss. Louder than a TV, quieter than a car.
    // Source: <https://www.epa.gov/archive/epa/aboutepa/epa-identifies-noise-levels-affecting-health-and-welfare.html>
    const MAX_DB: f32 = 70.0;
    let fraction = (db_spl - MIN_DB) / (MAX_DB - MIN_DB);
    fraction.clamp(0.0, 1.0)
}
