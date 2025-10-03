use bevy::{platform::collections::HashMap, prelude::*, time::Stopwatch};
use evergreen_relations::prelude::*;

use crate::demo::ai::sense::SenseTimer;

pub(crate) mod __many_to_many;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(__many_to_many::plugin);
    app.add_systems(PreUpdate, tick_awareness_times);
}

#[derive(Debug, Component, Default)]
#[require(AwarenessData, SenseTimer)]
pub(crate) struct Alertness(pub(crate) AwarenessLevel);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum AwarenessLevel {
    #[default]
    Lowest = 0,
    Low,
    Moderate,
    High,
}

#[derive(Debug, Component, Default, Deref, DerefMut)]
pub(crate) struct AwarenessData(HashMap<Entity, Awareness>);

#[derive(Debug, Default)]
pub(crate) struct Awareness {
    pub(crate) level: AwarenessLevel,
    /// Time in current [`Awareness::level`]
    pub(crate) time: Stopwatch,
}

impl Awareness {
    #[expect(dead_code)]
    pub(crate) fn set_level(&mut self, level: AwarenessLevel) {
        self.level = level;
        self.time.reset();
    }
}

/// Undirected n:m relation between AIs and what they are aware of
pub(crate) type AwarenessLink = Related<__many_to_many::__AwarenessRelatable>;

fn tick_awareness_times(mut data: Query<&mut AwarenessData>, time: Res<Time>) {
    for mut awareness_data in data.iter_mut() {
        for (_, awareness) in awareness_data.iter_mut() {
            awareness.time.tick(time.delta());
        }
    }
}
