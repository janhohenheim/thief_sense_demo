//! Spawn the main level.

use bevy::prelude::*;
use bevy_landmass::prelude::*;
use bevy_rerecast::Navmesh;
use bevy_steam_audio::probes::GenerateProbes;
use bevy_trenchbroom::physics::SceneCollidersReady;
use landmass_rerecast::{Island3dBundle, NavMeshHandle3d};

use crate::{
    asset_tracking::LoadResource,
    demo::{npc::NPC_RADIUS, path_corner::link_path_corners, target::link_targets},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_asset::<Scene>(MAP).load_asset::<Navmesh>(NAVMESH);
}

const MAP: &str = "maps/main_level.map#Scene";
const NAVMESH: &str = "maps/main_level.nav";

/// A system that spawns the main level.
pub(crate) fn spawn_level(mut commands: Commands, assets: Res<AssetServer>) {
    commands
        .spawn((
            Name::new("Main Level"),
            SceneRoot(assets.load(MAP)),
            DespawnOnExit(Screen::Gameplay),
        ))
        .observe(link_path_corners)
        .observe(link_targets)
        .observe(make_brushes_pickable)
        .observe(bake_probes);
    let archipelago = commands
        .spawn((
            Name::new("Main Level Archipelago"),
            DespawnOnExit(Screen::Gameplay),
            Archipelago3d::new(ArchipelagoOptions::from_agent_radius(NPC_RADIUS)),
        ))
        .id();

    commands.spawn((
        Name::new("Main Level Island"),
        DespawnOnExit(Screen::Gameplay),
        Island3dBundle {
            island: Island,
            archipelago_ref: ArchipelagoRef3d::new(archipelago),
            nav_mesh: NavMeshHandle3d(assets.load(NAVMESH)),
        },
    ));
}

fn bake_probes(_ready: On<SceneCollidersReady>, mut generate: MessageWriter<GenerateProbes>) {
    generate.write(GenerateProbes {
        spacing: 2.0,
        height: 1.0,
        ..default()
    });
}

fn make_brushes_pickable(
    ready: On<SceneCollidersReady>,
    children: Query<&Children>,
    // contains only brushes at this point; entities are not spawned yet (I think)
    meshes: Query<Entity, With<Mesh3d>>,
    mut commands: Commands,
) {
    for entity in meshes.iter_many(children.iter_descendants(ready.scene_root_entity)) {
        commands.entity(entity).insert(Pickable::default());
    }
}
