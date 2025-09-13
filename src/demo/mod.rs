//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

mod click;
pub(crate) mod level;
pub(crate) mod npc;
mod path_corner;
mod target;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        level::plugin,
        npc::plugin,
        click::plugin,
        path_corner::plugin,
        target::plugin,
    ));
}
