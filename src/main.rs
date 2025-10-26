// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod animation;
mod asset_tracking;
mod audio;
mod collision_layer;
mod cpu_lighting;
mod demo;
mod despawn;
#[cfg(feature = "dev")]
mod dev_tools;
mod link_head;
mod movement;
mod screens;
mod solid_color;
mod staggered_timer;
mod third_party;
use bevy::{
    color::palettes::tailwind,
    gltf::GltfPlugin,
    image::{ImageAddressMode, ImageSamplerDescriptor},
    log::{LogPlugin, tracing_subscriber::field::MakeExt},
    prelude::*,
};

use crate::{
    demo::player::Player, solid_color::SolidColorEnvironmentMapLight as _,
    third_party::ui_anchor::UiAnchorCamera,
};

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub(crate) struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins((
            DefaultPlugins
                .set(GltfPlugin {
                    use_model_forward_direction: true,
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        address_mode_w: ImageAddressMode::Repeat,
                        anisotropy_clamp: 8,
                        ..ImageSamplerDescriptor::linear()
                    },
                })
                .set(LogPlugin {
                    filter: format!(
                        concat!(
                            "{default},",
                            "symphonia_bundle_mp3::demuxer=warn,",
                            "symphonia_format_caf::demuxer=warn,",
                            "symphonia_format_isompf4::demuxer=warn,",
                            "symphonia_format_mkv::demuxer=warn,",
                            "symphonia_format_ogg::demuxer=warn,",
                            "symphonia_format_riff::demuxer=warn,",
                            "symphonia_format_wav::demuxer=warn,",
                            "calloop::loop_logic=error,",
                        ),
                        default = bevy::log::DEFAULT_FILTER
                    ),
                    fmt_layer: |_| {
                        Some(Box::new(
                            bevy::log::tracing_subscriber::fmt::Layer::default()
                                .without_time()
                                .map_fmt_fields(MakeExt::debug_alt)
                                .with_writer(std::io::stderr),
                        ))
                    },
                    ..default()
                }),
            MeshPickingPlugin,
            bevy_seedling::SeedlingPlugin::default(),
            #[cfg(feature = "dev")]
            (
                bevy::remote::RemotePlugin::default(),
                bevy::remote::http::RemoteHttpPlugin::default(),
            ),
        ));

        app.set_error_handler(bevy::ecs::error::error);

        app.insert_resource(AmbientLight::NONE);
        app.insert_resource(MeshPickingSettings {
            require_markers: true,
            ..default()
        })
        .insert_resource(UiPickingSettings {
            require_markers: true,
        });

        // Add other plugins.
        app.add_plugins((
            third_party::plugin,
            asset_tracking::plugin,
            audio::plugin,
            animation::plugin,
            demo::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            screens::plugin,
            staggered_timer::plugin,
            movement::plugin,
            cpu_lighting::plugin,
            solid_color::plugin,
            collision_layer::plugin,
            link_head::plugin,
            despawn::plugin,
        ));

        app.configure_sets(
            FixedPreUpdate,
            (
                GameFixedPreUpdateSystems::UpdateInputBuffers,
                GameFixedPreUpdateSystems::UpdateAccumulators,
            )
                .chain(),
        )
        .configure_sets(
            FixedUpdate,
            (
                GameFixedUpdateSystems::Senses.run_if(any_with_component::<Player>),
                GameFixedUpdateSystems::Despawn,
            )
                .chain(),
        )
        .configure_sets(
            RunFixedMainLoop,
            (GamePreFixedSystems::Bookkeep, GamePreFixedSystems::Commit)
                .chain()
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
        )
        .configure_sets(Update, (GameUpdateSystems::Animation).chain());

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum GameUpdateSystems {
    Animation,
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum GamePreFixedSystems {
    Bookkeep,
    /// Prepare simulators
    Commit,
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum GameFixedUpdateSystems {
    Senses,
    Despawn,
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum GameFixedPreUpdateSystems {
    UpdateInputBuffers,
    UpdateAccumulators,
}

fn spawn_camera(mut commands: Commands, mut image_assets: ResMut<Assets<Image>>) {
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        MeshPickingCamera,
        Transform::from_xyz(0.0, 10.0, 8.0).looking_to(Vec3::new(0.0, -1.0, -0.7), Vec3::Y),
        EnvironmentMapLight {
            intensity: 60.0,
            ..EnvironmentMapLight::solid_color(&mut image_assets, tailwind::AMBER_100.into())
        },
        UiAnchorCamera,
    ));
}
