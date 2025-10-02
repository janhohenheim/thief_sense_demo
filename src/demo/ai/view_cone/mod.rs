use avian3d::prelude::*;
use bevy::prelude::*;

mod collider;
pub(crate) mod debug;
mod default_values;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ViewCones>();
    app.add_plugins((collider::plugin, default_values::plugin, debug::plugin));
}

#[derive(Resource, Debug, Deref, DerefMut)]
pub(crate) struct ViewCones(pub(crate) Vec<ViewCone>);

#[derive(Debug)]
pub(crate) struct ViewCone {
    pub(crate) collider: Collider,
    pub(crate) acuity: f32,
    pub(crate) flags: ViewConeFlags,
}

bitflags::bitflags! {
    #[derive(Debug)]
    pub(crate) struct ViewConeFlags: u16 {
        const Active   =  0x01;
        const NoAlert0 =  0x02;
        const NoAlert1 =  0x04;
        const NoAlert2 =  0x08;
        const NoAlert3 =  0x10;

        const AlertnessRestricted = Self::NoAlert0.bits() | Self::NoAlert1.bits() | Self::NoAlert2.bits() | Self::NoAlert3.bits();

        const Periph   =  0x20;
        const Omni     =  0x40;
        const LowLight =  0x80;

        const Behind   = 0x100;
    }
}

struct VisibilityAcuities {
    normal: VisibilityAcuity,
    periphery: VisibilityAcuity,
    omnidirectional: VisibilityAcuity,
    light: VisibilityAcuity,
    movement: VisibilityAcuity,
    low_light: VisibilityAcuity,
}

struct VisibilityAcuity {
    lighting: f32,
    movement: f32,
    exposure: f32,
}
