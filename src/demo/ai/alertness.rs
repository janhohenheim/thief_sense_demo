use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

#[derive(Component, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub(crate) enum Alertness {
    #[default]
    Lowest = 0,
    Low,
    Moderate,
    High,
}
