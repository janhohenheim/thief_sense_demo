use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use bevy_steam_audio::{probes::SteamAudioProbeBatch, scene::SteamAudioRootScene};

use crate::{
    GamePreFixedSystems,
    demo::ai::hearing::{AiSimulator, AiSource, accumulator::AudioInputsReady, param},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, init_simulation).add_systems(
        RunFixedMainLoop,
        update_probe_batch.in_set(GamePreFixedSystems::Bookkeep),
    );
    app.add_observer(add_source).add_observer(remove_source);
}

fn init_simulation(mut simulator: ResMut<AiSimulator>, scene: Res<SteamAudioRootScene>) {
    simulator.set_scene(&scene.0);
    simulator.commit();
}

fn update_probe_batch(probes: If<Res<SteamAudioProbeBatch>>, mut simulator: ResMut<AiSimulator>) {
    if !probes.is_changed() {
        return;
    }
    simulator.add_probe_batch(&probes);
    simulator.commit();
}

fn add_source(
    ready: On<AudioInputsReady>,
    mut commands: Commands,
    simulator: ResMut<AiSimulator>,
) -> Result {
    let source = AiSource(
        audionimbus::Source::try_new(
            &simulator,
            &audionimbus::SourceSettings {
                flags: param::FLAGS,
            },
        )
        .unwrap(),
    );
    simulator.add_source(&source);

    commands.entity(ready.0).try_insert(source);
    Ok(())
}

fn remove_source(
    remove: On<Remove, AiSource>,
    sources: Query<&AiSource, Allow<Disabled>>,
    simulator: ResMut<AiSimulator>,
) -> Result {
    let source = sources.get(remove.entity)?;
    simulator.remove_source(source);

    Ok(())
}
