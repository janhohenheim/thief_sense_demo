use std::time::Duration;

use bevy::{
    ecs::{component::Mutable, entity_disabling::Disabled},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

#[derive(Debug)]
pub(crate) struct StaggeredTimer {
    timer: Timer,
    stagger: Option<Timer>,
    just_spawned: bool,
    base_duration: Duration,
}

impl StaggeredTimer {
    pub(crate) fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
            stagger: None,
            just_spawned: false,
            base_duration: duration,
        }
    }

    pub(crate) fn tick(&mut self, duration: Duration) {
        if self.just_spawned {
            // Don't count the elapsed time of the frame before we even spawned
            self.just_spawned = false;
            return;
        }
        let stagger = self.stagger.as_mut().unwrap();

        let stagger_tail = stagger.remaining();
        stagger.tick(duration);

        if stagger.is_finished() {
            // duration must be >= stagger_tail, or the timer would not have finished in the first place
            self.timer.tick(duration - stagger_tail);
        }
    }

    /// Whether the timer is finished. Will always return false if stagger is not finished.
    pub(crate) fn is_finished(&self) -> bool {
        self.stagger.as_ref().unwrap().is_finished() && self.timer.is_finished()
    }

    /// Reset the timer. Does not stagger again.
    pub(crate) fn reset_with(&mut self, duration: Duration) {
        self.timer.set_duration(duration);
        self.reset();
    }

    /// Reset the timer. Does not stagger again.
    pub(crate) fn reset(&mut self) {
        self.timer.reset();
    }

    fn late_init(&mut self, index: usize) {
        const TARGET_FRAME_TIME: f32 = 1.0 / 60.0;
        let max_slot = (self.timer.duration().as_secs_f32() / TARGET_FRAME_TIME) as usize;

        let slot = if max_slot == 0 { 0 } else { index % max_slot };
        let stagger_duration = Duration::from_secs_f32(slot as f32 * TARGET_FRAME_TIME);
        self.stagger = Some(Timer::new(stagger_duration, TimerMode::Once));
    }
}

pub(crate) trait StaggeredTimerApp {
    fn add_staggered_timer<
        T: Component<Mutability = Mutable> + core::ops::DerefMut<Target = StaggeredTimer> + Default,
    >(
        &mut self,
    ) -> &mut App;
}

impl StaggeredTimerApp for App {
    fn add_staggered_timer<
        T: Component<Mutability = Mutable> + core::ops::DerefMut<Target = StaggeredTimer> + Default,
    >(
        &mut self,
    ) -> &mut App {
        self.add_observer(
            |add: On<Add, T>, mut timers: Query<&mut T, Allow<Disabled>>| {
                let timer_count = timers.count();
                // Unwrapping because otherwise we will panic later anyways. Better to panic earlier if this fails.
                let mut timer = timers.get_mut(add.entity).unwrap();
                timer.late_init(timer_count);
            },
        );
        self
    }
}
