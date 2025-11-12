use std::time::Duration;

use bevy::prelude::*;
use rand_distr::{Distribution as _, LogNormal};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

#[derive(Debug)]
pub(crate) struct LogNormalTimer {
    mean: f32,
    cv: f32,
    timer: Timer,
    just_spawned: bool,
}

impl LogNormalTimer {
    pub(crate) fn new(mean: f32, cv: f32) -> Self {
        let mut timer = Self {
            mean,
            cv,
            timer: Timer::default(),
            just_spawned: true,
        };
        timer.reset();
        timer
    }

    pub(crate) fn tick(&mut self, duration: Duration) {
        if self.just_spawned {
            // Don't count the elapsed time of the frame before we even spawned
            self.just_spawned = false;
            return;
        }
        self.timer.tick(duration);
    }

    /// Whether the timer is finished. Will always return false if stagger is not finished.
    pub(crate) fn is_finished(&self) -> bool {
        self.timer.is_finished()
    }

    #[expect(dead_code)]
    pub(crate) fn remaining(&self) -> Duration {
        self.timer.remaining()
    }

    /// Reset the timer. Uses a new random offset.
    pub(crate) fn reset(&mut self) {
        let mut rng = rand::rng();
        let duration = LogNormal::from_mean_cv(self.mean, self.cv)
            .unwrap()
            .sample(&mut rng);
        self.timer = Timer::from_seconds(duration, TimerMode::Once);
    }
}
