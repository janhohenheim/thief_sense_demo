//! Spawn the main level.

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_landmass::{PointSampleDistance3d, prelude::*};
use bevy_rerecast::Navmesh;
use landmass_rerecast::{Island3dBundle, NavMeshHandle3d};

use crate::{asset_tracking::LoadResource, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.load_asset::<Scene>(MAP).load_asset::<Navmesh>(NAVMESH);
}

const MAP: &str = "maps/main_level.map#Scene";
const NAVMESH: &str = "maps/main_level.nav";

/// A system that spawns the main level.
pub fn spawn_level(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        SceneRoot(assets.load(MAP)),
        StateScoped(Screen::Gameplay),
        children![(
            DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::default().looking_to(Vec3::new(2.0, -6.0, -1.0), Vec3::Y)
        )],
    ));
    let archipelago = commands
        .spawn((
            Name::new("Main Level Archipelago"),
            StateScoped(Screen::Gameplay),
            Archipelago3d::new(AgentOptions {
                point_sample_distance: PointSampleDistance3d {
                    horizontal_distance: 0.6,
                    distance_above: 1.0,
                    distance_below: 1.0,
                    vertical_preference_ratio: 2.0,
                },
                ..AgentOptions::from_agent_radius(0.4)
            }),
        ))
        .id();

    commands.spawn((
        Name::new("Main Level Island"),
        StateScoped(Screen::Gameplay),
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
