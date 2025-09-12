use bevy::prelude::*;

mod avian;
mod landmass;
mod rerecast;
mod trenchbroom;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        trenchbroom::plugin,
        avian::plugin,
        rerecast::plugin,
        landmass::plugin,
    ));
}
