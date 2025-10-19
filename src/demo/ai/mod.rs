use bevy::prelude::*;

pub(crate) mod awareness;
pub(crate) mod hearing;
pub(crate) mod sense;
pub(crate) mod vision;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        sense::plugin,
        awareness::plugin,
        vision::plugin,
        hearing::plugin,
    ));
}
