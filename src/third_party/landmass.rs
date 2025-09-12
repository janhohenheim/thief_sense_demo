//! [Landmass](https://github.com/andriyDev/landmass) powers out agent navigation.
//! The underlying navmesh is generated using [Oxidized Navigation](https://github.com/TheGrimsey/oxidized_navigation).

use bevy::prelude::*;
use bevy_landmass::{debug::Landmass3dDebugPlugin, prelude::*};
use landmass_rerecast::LandmassRerecastPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        Landmass3dPlugin::default(),
        LandmassRerecastPlugin::default(),
        Landmass3dDebugPlugin::default(),
    ));
}
