use std::sync::atomic::Ordering;

use bevy::prelude::*;
use bevy_steam_audio::wrapper::AudionimbusCoordinateSystem;

use crate::{
    AppSystems,
    demo::ai::audio_simulation::{AiAsyncSimulationSynchronization, AiSimulator, AiSource, param},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_simulation.in_set(AppSystems::AudioSimulation),
    );
}

fn update_simulation(
    simulator: ResMut<AiSimulator>,
    synchro: ResMut<AiAsyncSimulationSynchronization>,
    mut sources: Query<(&mut AiSource, &GlobalTransform)>,
    mut expensive_timer: Local<Option<Timer>>,
    time: Res<Time>,
    mut errors: Local<Vec<String>>,
) -> Result {
    errors.clear();
    if synchro.complete.load(Ordering::SeqCst) {
        // This should never fail unless there's a bug, as this branch should only be called when the reflection thread is idle.
        simulator
            .try_write()
            .map_err(|e| format!("Failed to commit simulator even though it should be idle: {e}"))?
            .commit();
    }
    let simulator = simulator
        .try_read()
        .map_err(|e| format!("Failed to run simulator even though it should be idle: {e}"))?;

    let shared_inputs = audionimbus::SimulationSharedInputs {
        listener: default(),
        num_rays: param::REFLECT_RAYS,
        num_bounces: param::REFLECT_BOUNCES,
        duration: param::REFLECT_DURATION,
        order: param::ORDER,
        irradiance_min_distance: 1.0,
        pathing_visualization_callback: None,
    };
    simulator.set_shared_inputs(audionimbus::SimulationFlags::DIRECT, &shared_inputs);

    let gen_inputs = |orientation: AudionimbusCoordinateSystem| audionimbus::SimulationInputs {
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
        reflections_simulation: audionimbus::ReflectionsSimulationParameters::Convolution {
            baked_data_identifier: None,
        }
        .into(),
        pathing_simulation: None,
        source: orientation.to_audionimbus(),
    };

    for (mut source, transform) in &mut sources {
        let transform = transform.compute_transform();
        let orientation = AudionimbusCoordinateSystem::from_bevy_transform(transform);

        source.set_inputs(
            audionimbus::SimulationFlags::DIRECT,
            gen_inputs(orientation),
        );
    }

    simulator.run_direct();

    let expensive_timer =
        expensive_timer.get_or_insert_with(|| Timer::from_seconds(0.2, TimerMode::Once));
    expensive_timer.tick(time.delta());
    if !expensive_timer.is_finished() {
        // Not yet time to kick off expensive simulation
        return if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n").into())
        };
    }
    if !synchro.complete.load(Ordering::SeqCst) {
        // It's time, but the previous simulation is still running!
        return if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n").into())
        };
    }

    simulator.set_shared_inputs(param::EXPENSIVE_FLAGS, &shared_inputs);
    for (mut source, transform) in &mut sources {
        let transform = transform.compute_transform();
        let orientation = AudionimbusCoordinateSystem::from_bevy_transform(transform);

        source.set_inputs(param::EXPENSIVE_FLAGS, gen_inputs(orientation));
    }

    synchro.complete.store(false, Ordering::SeqCst);
    expensive_timer.reset();
    synchro.sender.send(())?;
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n").into())
    }
}
