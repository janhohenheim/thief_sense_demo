use bevy::prelude::*;
use bevy_trenchbroom::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TrenchBroomPlugins(TrenchBroomConfig::new(
        "Thief Sense Demo",
    )));
}
