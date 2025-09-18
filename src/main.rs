// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod animation;
mod asset_tracking;
mod audio;
mod demo;
#[cfg(feature = "dev")]
mod dev_tools;
mod screens;
mod theme;
mod third_party;
use bevy::{
    asset::AssetMetaCheck,
    gltf::GltfPlugin,
    image::{ImageAddressMode, ImageSamplerDescriptor},
    prelude::*,
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
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Thief Sense Demo".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
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
                }),
            MeshPickingPlugin,
            #[cfg(feature = "native")]
            bevy_seedling::SeedlingPlugin::default(),
            // right now, `Default` isn't implemented for any non-cpal backend
            #[cfg(feature = "web")]
            app.add_plugins(
                bevy_seedling::SeedlingPlugin::<firewheel_web_audio::WebAudioBackend> {
                    config: Default::default(),
                    stream_config: Default::default(),
                    spawn_default_pool: true,
                    pool_size: 4..=32,
                },
            ),
            #[cfg(feature = "dev_native")]
            (
                bevy::remote::RemotePlugin::default(),
                bevy::remote::http::RemoteHttpPlugin::default(),
            ),
        ));

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
            theme::plugin,
        ));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 8.0).looking_to(Vec3::new(0.0, -1.0, -0.7), Vec3::Y),
    ));
}
