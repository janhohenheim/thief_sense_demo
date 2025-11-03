use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use std::time::Duration;
use strum::EnumCount as _;

use crate::demo::{ai::awareness::AwarenessLevel, npc::Npc};

pub(super) fn plugin(app: &mut App) {
    app.register_required_components::<Npc, Alertness>();
    app.register_required_components::<Npc, FreeKnowledgeDurations>();
}

pub(crate) fn update_alertness(In(npc): In<Entity>) -> Result {
    Ok(())
}

#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
#[require(FreeKnowledgeDurations)]
#[component(on_add = Alertness::on_add)]
pub(crate) struct Alertness {
    pub(crate) level: AwarenessLevel,
    pub(crate) free_knowledge: Duration,
}

impl Default for Alertness {
    fn default() -> Self {
        Self {
            level: AwarenessLevel::default(),
            free_knowledge: FreeKnowledgeDurations::default()[AwarenessLevel::default() as usize],
        }
    }
}

impl Alertness {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let durations = *world.get::<FreeKnowledgeDurations>(ctx.entity).unwrap();
        let mut alertness = world.get_mut::<Alertness>(ctx.entity).unwrap();
        alertness.free_knowledge = durations[alertness.level as usize];
    }
}

#[derive(Debug, Component, Clone, Copy, Reflect, Deref, DerefMut)]
#[reflect(Component)]
// Note: original also multiplies this by 1.666666 when in combat.
pub(crate) struct FreeKnowledgeDurations([Duration; AwarenessLevel::COUNT]);

impl Default for FreeKnowledgeDurations {
    fn default() -> Self {
        Self([
            Duration::from_millis(1500), // 1.0
            Duration::from_millis(1500), // 1.0
            Duration::from_millis(1875), // 1.25
            Duration::from_millis(3000), // 2.0
        ])
    }
}
