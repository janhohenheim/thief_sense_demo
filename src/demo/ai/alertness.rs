use bevy::prelude::*;

use crate::demo::{ai::awareness::AwarenessLevel, npc::Npc};

pub(super) fn plugin(app: &mut App) {
    app.register_required_components::<Npc, Alertness>();
}

#[derive(Debug, Component, Default)]
pub(crate) struct Alertness(pub(crate) AwarenessLevel);
