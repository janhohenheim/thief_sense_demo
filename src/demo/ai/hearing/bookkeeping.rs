use std::{
    array,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use bevy::{ecs::entity_disabling::Disabled, prelude::*, tasks::AsyncComputeTaskPool};
use bevy_steam_audio::{scene::SteamAudioRootScene, sources::AudionimbusSource};

use crate::demo::ai::hearing::{AiAudible, AiSimulators, AiSources, param};

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
    let mut simulators = simulators.iter();
    let sources = array::from_fn(|_| {
        let simulator = simulators.next().unwrap();
        let source = audionimbus::Source::try_new(
            &simulator,
            &audionimbus::SourceSettings {
                flags: param::FLAGS,
            },
        )
        .unwrap();
        simulator.add_source(&source);
        source
    });
    commands.entity(add.entity).try_insert(AiSources(sources));
    Ok(())
}

fn sync_source_removal(remove: On<Remove, AudionimbusSource>, mut commands: Commands) {
    commands.entity(remove.entity).try_remove::<AiSources>();
}

fn remove_source(
    remove: On<Remove, AiSources>,
    source: Query<&AiSources, Allow<Disabled>>,
    simulators: ResMut<AiSimulators>,
) -> Result {
    let sources = source.get(remove.entity)?;
    for (simulator, source) in simulators.iter().zip(sources.iter()) {
        simulator.remove_source(source);
    }
    Ok(())
}
