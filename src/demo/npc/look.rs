use bevy::prelude::*;

use crate::demo::{npc::sense::SenseTimer, player::Player};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        look.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
    );
}

fn look(npcs: Query<&SenseTimer>, player: Single<&Transform, With<Player>>) {
    for sense_timer in &npcs {
        if !sense_timer.is_finished() {
            continue;
        }
    }
}
