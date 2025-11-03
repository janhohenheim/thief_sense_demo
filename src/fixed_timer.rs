use std::ops::DerefMut;

use bevy::{ecs::component::Mutable, prelude::*};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

pub(crate) trait FixedTimerApp {
    fn add_fixed_timer<T: DerefMut<Target = Timer> + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self;
}

impl FixedTimerApp for App {
    fn add_fixed_timer<T: DerefMut<Target = Timer> + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self {
        self.add_systems(
            FixedPreUpdate,
            |mut timers: Query<&mut T>, time: Res<Time>| {
                for mut timer in &mut timers {
                    timer.deref_mut().tick(time.delta());
                }
            },
        );
        self
    }
}
