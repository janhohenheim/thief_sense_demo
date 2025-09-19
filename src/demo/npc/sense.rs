use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, tick_timers);
}

#[derive(Component, Debug)]
#[component(on_add = on_add_npc_sense_timer)]
pub(crate) struct NpcSenseTimer {
    pub(crate) timer: Timer,
    pub(crate) max_offset: f32,
    pub(crate) base_duration: f32,
}

impl Default for NpcSenseTimer {
    fn default() -> Self {
        Self {
            max_offset: 0.05,
            base_duration: 0.2,
            // set in hook
            timer: Timer::default(),
        }
    }
}

impl NpcSenseTimer {
    pub(crate) fn reset(&mut self) {
        self.timer = Timer::from_seconds(
            self.base_duration + rand::random_range(-self.max_offset..self.max_offset),
            TimerMode::Once,
        );
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.timer.is_finished()
    }
}

#[derive(Component, Debug)]
#[component(on_add = on_add_player_sense_timer)]
pub(crate) struct PlayerSenseTimer {
    pub(crate) timer: Timer,
    pub(crate) max_offset: f32,
    pub(crate) near_duration: f32,
    pub(crate) far_duration: f32,
    pub(crate) far_distance: f32,
}

impl Default for PlayerSenseTimer {
    fn default() -> Self {
        Self {
            max_offset: 0.05,
            near_duration: 0.2,
            far_duration: 0.5,
            far_distance: 15.0,
            // set in hook
            timer: Timer::default(),
        }
    }
}

impl PlayerSenseTimer {
    pub(crate) fn reset_near(&mut self) {
        self.timer = Timer::from_seconds(
            self.near_duration + rand::random_range(-self.max_offset..self.max_offset),
            TimerMode::Once,
        );
    }

    pub(crate) fn reset_far(&mut self) {
        self.timer = Timer::from_seconds(
            self.far_duration + rand::random_range(-self.max_offset..self.max_offset),
            TimerMode::Once,
        );
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.timer.is_finished()
    }
}

fn on_add_npc_sense_timer(mut world: DeferredWorld, ctx: HookContext) {
    world
        .entity_mut(ctx.entity)
        .get_mut::<NpcSenseTimer>()
        .unwrap()
        .reset();
}

fn on_add_player_sense_timer(mut world: DeferredWorld, ctx: HookContext) {
    world
        .entity_mut(ctx.entity)
        .get_mut::<PlayerSenseTimer>()
        .unwrap()
        .reset_far();
}

fn tick_timers(
    mut npc_timers: Query<&mut NpcSenseTimer>,
    mut player_timers: Query<&mut PlayerSenseTimer>,
    time: Res<Time>,
) {
    for mut timer in npc_timers.iter_mut() {
        timer.timer.tick(time.delta());
    }

    for mut timer in player_timers.iter_mut() {
        timer.timer.tick(time.delta());
    }
}
