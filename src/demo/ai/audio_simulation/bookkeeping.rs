use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bevy::{ecs::entity_disabling::Disabled, prelude::*, tasks::AsyncComputeTaskPool};
use bevy_steam_audio::{scene::SteamAudioRootScene, sources::AudionimbusSource};

use crate::demo::ai::audio_simulation::{
    AiAsyncSimulationSynchronization, AiAudible, AiSimulator, AiSource, param,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, init_simulation);
    app.add_observer(add_source)
        .add_observer(remove_source)
        .add_observer(sync_source_removal);
}

fn init_simulation(
    mut commands: Commands,
    simulator: Res<AiSimulator>,
    scene: Res<SteamAudioRootScene>,
) {
    {
        let mut simulator = simulator
            .try_write()
            .expect("The simulator should not be in use during initialization");
        simulator.set_scene(&scene.0);
        simulator.commit();
    }
    let simulation_complete = Arc::new(AtomicBool::new(false));
    let simulation_complete_inner = simulation_complete.clone();
    let (tx, rx) = crossbeam_channel::unbounded::<()>();
    commands.insert_resource(AiAsyncSimulationSynchronization {
        sender: tx,
        complete: simulation_complete,
    });

    let simulator = simulator.clone();
    let future = async move {
        loop {
            {
                // Block thread until simulator is ready
                let simulator = simulator.read().unwrap();
                simulator.run_reflections();
            }

            simulation_complete_inner.store(true, Ordering::Relaxed);
            if rx.recv().is_err() {
                // tx dropped because we created a new simulation
                break;
            }
        }
    };
    AsyncComputeTaskPool::get().spawn(future).detach();
}

fn add_source(
    add: On<Add, AudionimbusSource>,
    ai_audible: Query<(), (With<AiAudible>, Allow<Disabled>)>,
    mut commands: Commands,
    simulator: ResMut<AiSimulator>,
) -> Result {
    if !ai_audible.contains(add.entity) {
        return Ok(());
    }
    let simulator = simulator.try_read().map_err(|err| {
        format!("The simulator should never be written in parallel to being read, but got {err}")
    })?;
    let source = audionimbus::Source::try_new(
        &simulator,
        &audionimbus::SourceSettings {
            flags: param::FLAGS,
        },
    )?;
    simulator.add_source(&source);
    commands.entity(add.entity).try_insert(AiSource(source));
    Ok(())
}

fn sync_source_removal(remove: On<Remove, AudionimbusSource>, mut commands: Commands) {
    commands.entity(remove.entity).try_remove::<AiSource>();
}

fn remove_source(
    remove: On<Remove, AiSource>,
    source: Query<&AiSource, Allow<Disabled>>,
    simulator: ResMut<AiSimulator>,
) -> Result {
    let source = source.get(remove.entity)?;
    let simulator = simulator.try_read().map_err(|err| {
        format!("The simulator should never be written in parallel to being read, but got {err}")
    })?;
    simulator.remove_source(source);
    Ok(())
}
