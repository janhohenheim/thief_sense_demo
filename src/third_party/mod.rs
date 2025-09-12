use bevy::prelude::*;

mod avian;
mod bevy_trenchbroom;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((bevy_trenchbroom::plugin, avian::plugin));
}
