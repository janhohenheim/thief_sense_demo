use avian3d::prelude::*;
use bevy::prelude::*;

use crate::demo::ai::alertness::Alertness;

mod collider;
pub(crate) mod debug;
mod default_values;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ViewCones>();
    app.init_resource::<VisibilityAcuities>();
    app.add_plugins((collider::plugin, default_values::plugin, debug::plugin));
}

#[derive(Resource, Debug, Deref, DerefMut)]
pub(crate) struct ViewCones(pub(crate) Vec<ViewCone>);

#[derive(Debug, Clone)]
pub(crate) struct ViewCone {
    pub(crate) collider: Collider,
    pub(crate) acuity: f32,
    pub(crate) flags: ViewConeFlags,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
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

impl ViewConeFlags {
    pub(crate) fn active(&self) -> bool {
        self.contains(Self::Active)
    }

    pub(crate) fn allowed_by(self, alertness: Alertness) -> bool {
        if self.contains(Self::NoAlert0) && alertness == Alertness::Lowest {
            return false;
        }
        if self.contains(Self::NoAlert1) && alertness == Alertness::Low {
            return false;
        }
        if self.contains(Self::NoAlert2) && alertness == Alertness::Moderate {
            return false;
        }
        if self.contains(Self::NoAlert3) && alertness == Alertness::High {
            return false;
        }
        true
    }
}

#[derive(Debug, Resource, Clone, Copy)]
pub(crate) struct VisibilityAcuities {
    pub(crate) normal: VisibilityAcuity,
    pub(crate) periphery: VisibilityAcuity,
    pub(crate) omnidirectional: VisibilityAcuity,
    pub(crate) low_light: VisibilityAcuity,
}

impl VisibilityAcuities {
    pub(crate) fn for_cone(self, cone: ViewConeFlags) -> VisibilityAcuity {
        if cone.contains(ViewConeFlags::LowLight) {
            self.low_light
        } else if cone.contains(ViewConeFlags::Periph) {
            self.periphery
        } else if cone.contains(ViewConeFlags::Omni) {
            self.omnidirectional
        } else {
            self.normal
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct VisibilityAcuity {
    pub(crate) lighting: f32,
    pub(crate) movement: f32,
    pub(crate) exposure: f32,
}
