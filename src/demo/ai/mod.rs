use bevy::prelude::*;

mod look;
pub(crate) mod sense;
pub(crate) mod view_cone;
pub(crate) mod visibility;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        look::plugin,
        sense::plugin,
        view_cone::plugin,
        visibility::plugin,
    ));
}
