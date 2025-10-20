use bevy::prelude::*;
use bevy_steam_audio::{
    probes::SteamAudioProbeBatch, settings::SteamAudioPathBakingSettings,
    wrapper::AudionimbusCoordinateSystem,
};

use crate::demo::ai::hearing::{AiSimulators, AiSource, param};

pub(super) fn plugin(_app: &mut App) {}

pub(crate) struct AiSimulationInputs {
    pub(crate) listener: Entity,
    pub(crate) sources: Vec<Entity>,
}

pub(crate) fn update_simulation_for_listener(
    In(AiSimulationInputs {
        listener,
        sources: source_entities,
    }): In<AiSimulationInputs>,
    mut simulator: ResMut<AiSimulators>,
    mut sources: Query<(&mut AiSource, &GlobalTransform)>,
    mut errors: Local<Vec<String>>,
    transform: Query<&GlobalTransform>,
    batch: Res<SteamAudioProbeBatch>,
    path_baking_settings: Res<SteamAudioPathBakingSettings>,
) -> Result {
    errors.clear();
    simulator.commit();
    let listener = transform.get(listener)?;
    let listener = AudionimbusCoordinateSystem::from(*listener);

    simulator.set_shared_inputs(
        param::FLAGS,
        &audionimbus::SimulationSharedInputs {
            listener: listener.into(),
            num_rays: 0,
            num_bounces: 0,
            duration: 0.0,
            order: param::ORDER,
            irradiance_min_distance: 1.0,
            pathing_visualization_callback: None,
        },
    );

    let mut sources = sources.iter_many_mut(source_entities);
    while let Some((mut source, transform)) = sources.fetch_next() {
        source.set_inputs(
            param::FLAGS,
            audionimbus::SimulationInputs {
                direct_simulation: audionimbus::DirectSimulationParameters {
                    distance_attenuation: audionimbus::DistanceAttenuationModel::Default.into(),
                    air_absorption: audionimbus::AirAbsorptionModel::Default.into(),
                    // TODO: actually ask the source for this, once bevy_steam_audio supports it
                    directivity: audionimbus::Directivity::WeightedDipole {
                        weight: 0.0,
                        power: 0.0,
                    }
                    .into(),
                    occlusion: audionimbus::Occlusion {
                        transmission: audionimbus::TransmissionParameters {
                            num_transmission_rays: 4,
                        }
                        .into(),
                        algorithm: audionimbus::OcclusionAlgorithm::Raycast,
                    }
                    .into(),
                }
                .into(),
                reflections_simulation: None,
                pathing_simulation: audionimbus::PathingSimulationParameters {
                    pathing_probes: &batch.0,
                    visibility_radius: path_baking_settings.visibility_radius,
                    visibility_threshold: path_baking_settings.visibility_threshold,
                    visibility_range: path_baking_settings.visibility_range,
                    pathing_order: param::ORDER,
                    enable_validation: true,
                    find_alternate_paths: true,
                    deviation: audionimbus::DeviationModel::Default,
                }
                .into(),
                source: AudionimbusCoordinateSystem::from(*transform).into(),
            },
        );
    }

    simulator.run_direct();
    simulator.run_pathing();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n").into())
    }
}
