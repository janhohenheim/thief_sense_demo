use bevy::prelude::*;

use crate::demo::ai::{
    awareness::AwarenessLevel,
    debug::DebugHearing,
    hearing::{
        AiSources,
        loudness::{LoudnessInput, loudness_to_listener},
        simulate::{AiSimulationInputs, update_simulation_for_listener},
    },
};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

pub(crate) fn listen(
    In((npc, near)): In<(Entity, bool)>,
    world: &mut World,
) -> Result<Vec<(Entity, AwarenessLevel)>> {
    let sources: Vec<_> = world.run_system_cached_with(sources_for_listener, npc)?;

    () = world.run_system_cached_with(
        update_simulation_for_listener,
        AiSimulationInputs {
            listener: npc,
            sources: sources.clone(),
            near,
        },
    )?;
    let mut pulses = Vec::new();
    let mut total_loudness = 0.0;
    for source in sources {
        let raw_loudness: f32 = world.run_system_cached_with(
            loudness_to_listener,
            LoudnessInput {
                listener: npc,
                source,
                near,
            },
        )?;
        total_loudness += raw_loudness;
        info!("{raw_loudness:0.8}");
        let loudness = raw_loudness as u32;

        // TODO: This is just placeholder code. It's fine, but the loudness is still raw and not factoring in any attenuation factors or object "mod" factors.
        let pulse = match loudness {
            v if v < 25 => AwarenessLevel::Lowest,
            v if v < 50 => AwarenessLevel::Low,
            v if v < 75 => AwarenessLevel::Moderate,
            _ => AwarenessLevel::High,
        };
        pulses.push((source, pulse));
    }
    info!("");
    world.entity_mut(npc).insert(DebugHearing(total_loudness));
    Ok(pulses)
}

fn sources_for_listener(
    In(npc): In<Entity>,
    transform: Query<&GlobalTransform>,
    sources: Query<(Entity, &GlobalTransform), With<AiSources>>,
) -> Result<Vec<Entity>> {
    let npc_translation = transform.get(npc)?.translation();
    let sources = sources
        .iter()
        .filter_map(|(entity, transform)| {
            if transform.translation().distance_squared(npc_translation) < 100.0 * 100.0 {
                Some(entity)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Ok(sources)
}
