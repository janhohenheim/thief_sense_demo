use avian3d::prelude::*;
use bevy::prelude::*;

pub(crate) mod trimesh;
pub(crate) mod view_cone;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default());
}
