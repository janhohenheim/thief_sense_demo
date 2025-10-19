use bevy::prelude::*;
use bevy_steam_audio::SteamAudioListener;

use crate::{demo::player::Player, link_head::Head};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_listener);
    app.add_systems(Update, sync_listener_transform);
}

fn spawn_listener(mut commands: Commands) {
    commands.spawn((Name::new("Listener"), SteamAudioListener));
}

fn sync_listener_transform(
    mut listener: Single<&mut Transform, With<SteamAudioListener>>,
    player_head: Single<&Head, With<Player>>,
    transform: Query<&GlobalTransform>,
) -> Result {
    let head = player_head.into_inner().iter().next().unwrap();
    let head_transform = transform.get(head)?;
    listener.translation = head_transform.translation();
    Ok(())
}
