use bevy::prelude::*;

use crate::demo::{npc::Npc, player::Player};

pub(super) fn plugin(app: &mut App) {
    app.register_required_components::<Npc, Team>()
        .register_required_components::<Player, Team>();
}

#[derive(Component, Clone, Copy, Reflect, Debug, PartialEq, Eq)]
#[reflect(Component)]
pub(crate) enum Team {
    Good,
    Neutral,
    Bad(u8),
    Alarm,
}

impl Default for Team {
    fn default() -> Self {
        panic!("Need to set the team explicitly")
    }
}

impl Team {
    pub(crate) fn relation_to(self, other: Self) -> TeamRelation {
        if self == Team::Alarm || other == Team::Alarm {
            TeamRelation::Enemy
        } else if self == Team::Neutral || other == Team::Neutral {
            TeamRelation::Indifferent
        } else if self == other {
            TeamRelation::Ally
        } else {
            TeamRelation::Enemy
        }
    }
}

#[derive(Clone, Copy, Reflect, Debug, PartialEq, Eq)]
pub(crate) enum TeamRelation {
    Indifferent,
    Ally,
    Enemy,
}
