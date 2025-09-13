//! [Landmass](https://github.com/andriyDev/landmass) powers out agent navigation.
//! The underlying navmesh is generated using [Oxidized Navigation](https://github.com/TheGrimsey/oxidized_navigation).

use bevy::prelude::*;
use bevy_landmass::{debug::Landmass3dDebugPlugin, prelude::*};
use landmass_rerecast::LandmassRerecastPlugin;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Agent>();
    app.register_type::<AgentOf>();
    app.add_plugins((
        Landmass3dPlugin::default(),
        LandmassRerecastPlugin::default(),
        Landmass3dDebugPlugin::default(),
    ));
}

#[derive(Component, Deref, Debug, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = Agent)]
pub(crate) struct AgentOf(pub(crate) Entity);

#[derive(Component, Deref, Debug, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = AgentOf)]
pub(crate) struct Agent(Entity);
