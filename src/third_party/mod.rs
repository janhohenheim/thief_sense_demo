use bevy::prelude::*;

pub(crate) mod avian;
pub(crate) mod landmass;
mod rerecast;
mod tnua;
mod trenchbroom;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        trenchbroom::plugin,
        avian::plugin,
        rerecast::plugin,
        landmass::plugin,
        tnua::plugin,
    ));
}
