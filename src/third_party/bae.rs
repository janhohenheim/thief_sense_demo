use bevy::prelude::*;
use bevy_bae::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(BaePlugin::default());
}
