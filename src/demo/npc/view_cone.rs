use crate::third_party::avian::EllipticCone as _;
use avian3d::prelude::*;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ViewCones>();
}

#[derive(Resource, Debug, Deref, DerefMut)]
pub(crate) struct ViewCones(pub(crate) Vec<ViewCone>);

impl FromWorld for ViewCones {
    fn from_world(_world: &mut World) -> Self {
        Self(vec![ViewCone {
            collider: Collider::elliptic_cone(0.4, 0.8, 3.5),
        }])
    }
}

#[derive(Debug)]
pub(crate) struct ViewCone {
    pub(crate) collider: Collider,
}
