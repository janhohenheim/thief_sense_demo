use avian_steam_audio::AvianSteamAudioScenePlugin;
use bevy::prelude::*;
use bevy_steam_audio::prelude::*;
use trenchbroom_steam_audio::TrenchBroomSteamAudioScenePlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        SteamAudioPlugin::default(),
        AvianSteamAudioScenePlugin,
        TrenchBroomSteamAudioScenePlugin,
    ));
}
