//! The screen state for the main gameplay.

use bevy::prelude::*;

use crate::{demo::level::spawn_level, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);
}
