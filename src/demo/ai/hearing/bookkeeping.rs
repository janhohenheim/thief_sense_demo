use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use bevy_steam_audio::{
    probes::SteamAudioProbeBatch, scene::SteamAudioRootScene, sources::AudionimbusSource,
};

use crate::{
    AiSystems,
    demo::ai::hearing::{AiAudible, AiSimulators, AiSources, Simulator, param},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, init_simulation).add_systems(
        RunFixedMainLoop,
        update_probe_batch.in_set(AiSystems::Bookkeep),
    );
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

fn update_probe_batch(probes: Res<SteamAudioProbeBatch>, mut simulators: ResMut<AiSimulators>) {
    if !probes.is_changed() {
        return;
    }
    for simulator in simulators.iter_mut() {
        simulator.add_probe_batch(&probes);
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
    let source = |simulator: &Simulator| {
        audionimbus::Source::try_new(
            simulator,
            &audionimbus::SourceSettings {
                flags: param::FLAGS,
            },
        )
        .unwrap()
    };
    let sources = AiSources {
        near: source(&simulators.near),
        far: source(&simulators.far),
    };
    simulators.near.add_source(&sources.near);
    simulators.far.add_source(&sources.far);

    commands.entity(add.entity).try_insert(sources);
    Ok(())
}

fn sync_source_removal(remove: On<Remove, AudionimbusSource>, mut commands: Commands) {
    commands.entity(remove.entity).try_remove::<AiSources>();
}

fn remove_source(
    remove: On<Remove, AiSources>,
    sources: Query<&AiSources, Allow<Disabled>>,
    simulator: ResMut<AiSimulators>,
) -> Result {
    let sources = sources.get(remove.entity)?;
    simulator.near.remove_source(&sources.near);
    simulator.far.remove_source(&sources.far);

    Ok(())
}
