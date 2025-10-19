use bevy::prelude::*;

mod playback;
//mod simulation;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        playback::plugin,
        //simulation::plugin
    ));
}
