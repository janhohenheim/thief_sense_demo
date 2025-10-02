use bevy::prelude::*;

pub(crate) mod avian;
mod framepace;
pub(crate) mod landmass;
mod rerecast;
mod tnua;
mod trenchbroom;
pub(crate) mod ui_anchor;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        trenchbroom::plugin,
        avian::plugin,
        rerecast::plugin,
        landmass::plugin,
        tnua::plugin,
        framepace::plugin,
        ui_anchor::plugin,
    ));
}
