//! The screen state for the main gameplay.

use bevy::{input::common_conditions::input_just_pressed, prelude::*, ui::Val::*};

use crate::{Pause, demo::level::spawn_level, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);
}
