use bevy::prelude::*;
use bevy_trill::{LoadResponseEngine, TrillPlugin};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TrillPlugin)
        .add_systems(Startup, init_trill);
}

fn init_trill(mut commands: Commands) {
    commands.write_message(LoadResponseEngine::default().add_source_path("dialog/dialog.trl"));
}
