//! The game's main screen states and transitions between them.

mod gameplay;
mod loading;
mod splash;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((gameplay::plugin, loading::plugin, splash::plugin));
}

/// The game's main screen states.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub(crate) enum Screen {
    #[default]
    Splash,
    Loading,
    Gameplay,
}
