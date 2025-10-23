use bevy::prelude::*;
use bevy_steam_audio::{
    probes::SteamAudioProbeBatch, settings::SteamAudioPathBakingSettings,
    wrapper::AudionimbusCoordinateSystem,
};

use crate::{
    AiSystems,
    demo::ai::hearing::{
        AiSimulators, AiSources,
        debug::{PathVisualization, PathVisualizations, VisualizationsPtr},
        param,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        commit_simulators.in_set(AiSystems::Commit),
    );
}

pub(crate) struct AiSimulationInputs {
    pub(crate) listener: Entity,
    pub(crate) sources: Vec<Entity>,
    pub(crate) near: bool,
}

fn commit_simulators(mut simulators: ResMut<AiSimulators>) {
    for simulator in simulators.iter_mut() {
        simulator.commit();
    }
}

pub(crate) fn update_simulation_for_listener(
    In(AiSimulationInputs {
        listener,
        sources: source_entities,
        near,
    }): In<AiSimulationInputs>,
    simulators: ResMut<AiSimulators>,
    mut sources: Query<(&mut AiSources, &GlobalTransform)>,
    mut errors: Local<Vec<String>>,
    transform: Query<&GlobalTransform>,
    batch: Res<SteamAudioProbeBatch>,
    path_baking_settings: Res<SteamAudioPathBakingSettings>,
    visualizations: Res<PathVisualizations>,
) -> Result {
    errors.clear();
    let listener = transform.get(listener)?;
    let listener = AudionimbusCoordinateSystem::from(*listener);
    let simulator = if near {
        &simulators.near
    } else {
        &simulators.far
    };

    simulator.set_shared_inputs(
        param::FLAGS,
        &audionimbus::SimulationSharedInputs {
            listener: listener.into(),
            num_rays: 0,
            num_bounces: 0,
            duration: 0.0,
            order: param::ORDER,
            irradiance_min_distance: 1.0,
            pathing_visualization_callback: Some(audionimbus::CallbackInformation {
                callback: visualize_pathing,
                user_data: Box::into_raw(visualizations.0.clone()) as *mut std::ffi::c_void,
            }),
        },
    );

    let mut sources = sources.iter_many_mut(source_entities);
    while let Some((mut sources, transform)) = sources.fetch_next() {
        let source = if near {
            &mut sources.near
        } else {
            &mut sources.far
        };
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

unsafe extern "C" fn visualize_pathing(
    from: audionimbus_sys::IPLVector3,
    to: audionimbus_sys::IPLVector3,
    occluded: audionimbus_sys::IPLbool,
    user_data: *mut std::ffi::c_void,
) {
    let visualizations = user_data as VisualizationsPtr;
    // Safety: This is owned by the ECS and never deallocated
    let visualizations = &mut unsafe { (*visualizations).clone() };
    let visualization = PathVisualization::new(from, to, occluded);
    visualizations.lock().unwrap().push(visualization);
}
