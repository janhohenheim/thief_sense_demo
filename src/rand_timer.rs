use std::time::Duration;

use bevy::{ecs::component::Mutable, prelude::*};
use rand::Rng;

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

#[derive(Debug)]
pub(crate) struct RandTimer {
    timer: Timer,
    base_time: Duration,
}

impl RandTimer {
    pub(crate) fn from_millis(duration: u64) -> Self {
        Self::new(Duration::from_millis(duration))
    }

    pub(crate) fn from_secs_f32(duration: f32) -> Self {
        Self::new(Duration::from_secs_f32(duration))
    }

    pub(crate) fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(Self::offset(duration), TimerMode::Once),
            base_time: duration,
        }
    }

    pub(crate) fn tick(&mut self, time: impl AsRef<Time>) {
        self.timer.tick(time.as_ref().delta());
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.timer.is_finished()
    }

    pub(crate) fn finish(&mut self) {
        self.timer.finish();
    }

    /// Does *not* reset the timer
    pub(crate) fn set_base_time(&mut self, duration: Duration) {
        self.base_time = duration;
    }

    /// Does *not* reset the timer
    pub(crate) fn set_base_time_secs_f32(&mut self, duration: f32) {
        self.set_base_time(Duration::from_secs_f32(duration));
    }

    pub(crate) fn reset(&mut self) {
        self.timer = Timer::new(Self::offset(self.base_time), TimerMode::Once);
    }

    fn offset(duration: Duration) -> Duration {
        let base = duration.as_millis() as f32;
        let rand_duration = base + rand::rng().random_range(-1.0..1.0) * base * 0.1;
        Duration::from_millis(rand_duration as u64)
    }
}

pub(crate) trait RandTimerApp {
    fn add_rand_timer<
        T: Component<Mutability = Mutable> + core::ops::DerefMut<Target = RandTimer>,
    >(
        &mut self,
    ) -> &mut App;
}

impl RandTimerApp for App {
    fn add_rand_timer<
        T: Component<Mutability = Mutable> + core::ops::DerefMut<Target = RandTimer>,
    >(
        &mut self,
    ) -> &mut App {
        self.add_systems(PreUpdate, tick_rand_timer::<T>)
    }
}

fn tick_rand_timer<T: Component<Mutability = Mutable> + core::ops::DerefMut<Target = RandTimer>>(
    mut timers: Query<&mut T>,
    time: Res<Time>,
) {
    for mut timer in timers.iter_mut() {
        // doing the is_finished check before the tick so that the frame has time to read the is_finished state
        if timer.is_finished() {
            timer.reset();
        }
        timer.tick(&time);
    }
}
