use bevy::prelude::*;

pub(crate) mod avian;
mod bae;
mod framepace;
pub(crate) mod landmass;
mod rerecast;
mod steam_audio;
mod tnua;
mod trenchbroom;
mod trill;
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
        steam_audio::plugin,
        bae::plugin,
        trill::plugin,
    ));
}
