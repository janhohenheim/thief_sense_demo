use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_player);
}

#[derive(Component)]
pub(crate) struct Player;

fn spawn_player(mut commands: Commands) {
    commands.spawn((Player, Transform::default()));
}
