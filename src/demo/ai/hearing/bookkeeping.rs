use std::iter;

use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use bevy_steam_audio::{probes::SteamAudioProbeBatch, scene::SteamAudioRootScene};

use crate::{
    GameFixedPreUpdateSystems,
    demo::ai::hearing::{
        AiSimulator, AiSource, AiSourceBody, accumulator::InitSource, param, source_of::AiSourceOf,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, init_simulation).add_systems(
        FixedPreUpdate,
        update_probe_batch.in_set(GameFixedPreUpdateSystems::Bookkeep),
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
    ready: On<InitSource>,
    mut commands: Commands,
    has_source_already: Query<(), (With<AiSource>, Allow<Disabled>)>,
    simulator: ResMut<AiSimulator>,
    ai_source_bodies: Query<Entity, With<AiSourceBody>>,
    ancestors: Query<&ChildOf>,
) -> Result {
    if has_source_already.contains(ready.0) {
        return Ok(());
    };

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

    let body = ai_source_bodies
        .iter_many(iter::once(ready.0).chain(ancestors.iter_ancestors(ready.0)))
        .next()
        .ok_or("AiSource is not a descendant of an AiSourceBody")?;
    commands.entity(ready.0).try_insert(AiSourceOf { body });
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
