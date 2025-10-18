//! Reusable UI widgets & theming.

// Unused utilities may trigger this lints undesirably.
#![allow(dead_code)]

pub(crate) mod palette;

#[allow(unused_imports)]
pub(crate) mod prelude {
    pub(crate) use super::palette as ui_palette;
}

use bevy::prelude::*;

pub(super) fn plugin(_app: &mut App) {}
