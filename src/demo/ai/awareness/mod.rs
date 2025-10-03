use bevy::{platform::collections::HashMap, prelude::*};
use evergreen_relations::prelude::*;

use crate::demo::ai::sense::SenseTimer;

pub(crate) mod __many_to_many;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(__many_to_many::plugin);
}

#[derive(Debug, Component, Default)]
#[require(AwarenessData, SenseTimer)]
pub(crate) struct Alertness(pub(crate) AwarenessLevel);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub(crate) enum AwarenessLevel {
    #[default]
    Lowest = 0,
    Low,
    Moderate,
    High,
}

#[derive(Debug, Component, Default, Deref, DerefMut)]
pub(crate) struct AwarenessData(HashMap<Entity, AwarenessData>);

#[derive(Debug, Default)]
pub(crate) struct Awareness {
    // TODO: Awareness data associated with a specific object
}

/// Undirected n:m relation between AIs and what they are aware of
pub(crate) type AwarenessLink = Related<__many_to_many::__AwarenessRelatable>;
