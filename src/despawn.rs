use std::time::Duration;

use bevy::prelude::*;

use crate::GameFixedSystems;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, despawn.in_set(GameFixedSystems::Despawn));
}

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
#[reflect(Component)]
pub(crate) struct Despawn(pub(crate) Timer);

impl Despawn {
    pub fn after(secs: f32) -> Self {
        Self(Timer::new(Duration::from_secs_f32(secs), TimerMode::Once))
    }
}

impl Default for Despawn {
    fn default() -> Self {
        let mut timer = Timer::new(Duration::ZERO, TimerMode::Once);
        timer.finish();
        Self(timer)
    }
}

fn despawn(mut commands: Commands, mut to_despawn: Query<(Entity, &mut Despawn)>, time: Res<Time>) {
    for (entity, mut despawn) in to_despawn.iter_mut() {
        if despawn.is_finished() {
            commands.entity(entity).try_despawn();
        }
        despawn.tick(time.delta());
    }
}
