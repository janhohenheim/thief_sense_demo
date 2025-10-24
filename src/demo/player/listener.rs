use bevy::prelude::*;
use bevy_steam_audio::SteamAudioListener;

use crate::{demo::player::Player, link_head::Head};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_listener);
    app.add_systems(
        PostUpdate,
        sync_listener_transform.after(TransformSystems::Propagate),
    );
}

fn spawn_listener(mut commands: Commands) {
    commands.spawn((Name::new("Listener"), SteamAudioListener));
}

fn sync_listener_transform(
    listener: Single<(&mut Transform, &mut GlobalTransform), With<SteamAudioListener>>,
    player_head: Single<&Head, With<Player>>,
    transform: Query<&GlobalTransform, Without<SteamAudioListener>>,
) -> Result {
    let head = player_head.into_inner().iter().next().unwrap();
    let head_transform = transform.get(head)?;
    let (mut listener_transform, mut listener_global_transform) = listener.into_inner();
    listener_transform.translation = head_transform.translation();
    *listener_global_transform = GlobalTransform::from(*listener_transform);

    Ok(())
}
