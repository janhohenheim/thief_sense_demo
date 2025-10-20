use bevy::prelude::*;

pub(crate) mod look;
pub(crate) mod view_cone;
pub(crate) mod visibility;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((look::plugin, view_cone::plugin, visibility::plugin));
}
