use std::{
    array,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use bevy::{ecs::entity_disabling::Disabled, prelude::*, tasks::AsyncComputeTaskPool};
use bevy_steam_audio::{scene::SteamAudioRootScene, sources::AudionimbusSource};

use crate::demo::ai::hearing::{AiAudible, AiSimulators, AiSource, param};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, init_simulation);
    app.add_observer(add_source)
        .add_observer(remove_source)
        .add_observer(sync_source_removal);
}

fn init_simulation(mut simulators: ResMut<AiSimulators>, scene: Res<SteamAudioRootScene>) {
    for simulator in simulators.iter_mut() {
        simulator.set_scene(&scene.0);
        simulator.commit();
    }
}

fn add_source(
    add: On<Add, AudionimbusSource>,
    ai_audible: Query<(), (With<AiAudible>, Allow<Disabled>)>,
    mut commands: Commands,
    simulators: ResMut<AiSimulators>,
) -> Result {
    if !ai_audible.contains(add.entity) {
        return Ok(());
    }
    // All sims have the same settings
    let arbitrary_simulator = simulators.iter().next().unwrap();
    let source = audionimbus::Source::try_new(
        &arbitrary_simulator,
        &audionimbus::SourceSettings {
            flags: param::FLAGS,
        },
    )
    .unwrap();
    for simulator in simulators.iter() {
        simulator.add_source(&source);
    }

    commands.entity(add.entity).try_insert(AiSource(source));
    Ok(())
}

fn sync_source_removal(remove: On<Remove, AudionimbusSource>, mut commands: Commands) {
    commands.entity(remove.entity).try_remove::<AiSource>();
}

fn remove_source(
    remove: On<Remove, AiSource>,
    source: Query<&AiSource, Allow<Disabled>>,
    simulators: ResMut<AiSimulators>,
) -> Result {
    let source = source.get(remove.entity)?;
    for simulator in simulators.iter() {
        simulator.remove_source(source);
    }
    Ok(())
}
