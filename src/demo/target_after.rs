use bevy::prelude::*;

use crate::demo::target::Target;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, tick_timers);
    app.add_observer(remove_previous_target);
}

#[derive(Component)]
pub(crate) struct TargetAfter {
    pub(crate) target: Target,
    pub(crate) timer: Timer,
}

impl TargetAfter {
    pub(crate) fn new(target: Target, duration: f32) -> Self {
        Self {
            target,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

fn tick_timers(
    mut target_after: Query<(Entity, &mut TargetAfter)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut target_after) in target_after.iter_mut() {
        target_after.timer.tick(time.delta());
        if target_after.timer.is_finished() {
            commands
                .entity(entity)
                .remove::<TargetAfter>()
                .insert(target_after.target);
        }
    }
}

fn remove_previous_target(trigger: On<Add, TargetAfter>, mut commands: Commands) {
    commands.entity(trigger.entity).remove::<Target>();
}
