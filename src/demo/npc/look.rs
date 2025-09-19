use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        look.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
    );
}

fn look() {}
