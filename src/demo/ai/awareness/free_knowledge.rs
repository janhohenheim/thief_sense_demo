use std::time::Duration;

use bevy::prelude::*;
use strum::EnumCount;

use crate::demo::{ai::awareness::AwarenessLevel, npc::Npc};

pub(super) fn plugin(app: &mut App) {
    app.register_required_components::<Npc, FreeKnowledge>();
}

#[derive(Debug, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
// Note: original also multiplies this by 1.666666 when in combat.
pub(crate) struct FreeKnowledge([Duration; AwarenessLevel::COUNT]);

impl FreeKnowledge {
    pub(crate) fn get(&self, level: AwarenessLevel) -> Duration {
        self[level as usize]
    }
}

impl Default for FreeKnowledge {
    fn default() -> Self {
        Self([
            Duration::from_millis(1500), // 1.0
            Duration::from_millis(1500), // 1.0
            Duration::from_millis(1875), // 1.25
            Duration::from_millis(3000), // 2.0
        ])
    }
}
