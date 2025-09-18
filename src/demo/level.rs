//! Spawn the main level.

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_landmass::prelude::*;
use bevy_rerecast::Navmesh;
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
        .spawn((SceneRoot(assets.load(MAP)), DespawnOnExit(Screen::Gameplay)))
        .observe(link_path_corners)
        .observe(link_targets);
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
    commands.insert_resource(AmbientLight {
        color: tailwind::AMBER_200.into(),
        brightness: 150.0,
        ..default()
    });
}
