use bevy::prelude::*;

use crate::demo::ai::{
    awareness::AwarenessLevel,
    hearing::{
        AiSource,
        loudness::{LoudnessInput, loudness_to_listener},
        simulate::{AiSimulationInputs, update_simulation_for_listener},
    },
};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

pub(crate) fn listen(
    In(npc): In<Entity>,
    world: &mut World,
) -> Result<Vec<(Entity, AwarenessLevel)>> {
    let sources: Vec<_> = world.run_system_cached_with(sources_for_listener, npc)?;

    let _: () = world.run_system_cached_with(
        update_simulation_for_listener,
        AiSimulationInputs {
            listener: npc,
            sources: sources.clone(),
        },
    )?;
    let mut pulses = Vec::new();
    for source in sources {
        let raw_loudness: f32 = world.run_system_cached_with(
            loudness_to_listener,
            LoudnessInput {
                listener: npc,
                source,
            },
        )?;
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
    Ok(pulses)
}

fn sources_for_listener(
    In(npc): In<Entity>,
    transform: Query<&GlobalTransform>,
    sources: Query<(Entity, &GlobalTransform), With<AiSource>>,
) -> Result<Vec<Entity>> {
    let npc_translation = transform.get(npc)?.translation();
    // TODO: check_view_cones is the analogue for this function.
    // That one does the sense timer stuff. That probably should be "one step higher" to also cover this method here, right?
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
