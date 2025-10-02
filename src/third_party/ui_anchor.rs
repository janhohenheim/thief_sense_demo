use bevy::prelude::*;
use bevy_ui_anchor::AnchorUiPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AnchorUiPlugin::<UiAnchorCamera>::new());
}

#[derive(Component)]
pub(crate) struct UiAnchorCamera;
