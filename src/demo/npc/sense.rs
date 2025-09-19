//! Original implementation notes:
//! The AI iterates over the player and all close-ish NPCs. This represents the AI sensing the player and sensing other NPCs.
//! In every iteration, it checks a timer. For NPCs, it's 500 milliseconds. For the player, it's 200 milliseconds if near (about 12 meters), 500 milliseconds otherwise.
//! If the timer is due, the sensing happens. Vision is based on what the vision cones see *right now* this frame.
//! Only the highest order vision cone that contains the target is used.
//! Visibility is cached. Dunno if only the raycasts or more.
//! The sound meanwhile is buffered and considers all sounds that happened since the last time the timer was due.
//! Interestingly, all of this is only true for the AI sensing players and NPCs. Looking at suspicious objects is done completely separately, no vision cones involved.
//! Sound for e.g. thrown plates is also done separately, but I'm not sure of the timers used in both cases.

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::demo::player::Player;

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
    mut player_timers: Query<(&Transform, &mut PlayerSenseTimer)>,
    player: Single<&Transform, With<Player>>,
    time: Res<Time>,
) {
    for mut timer in npc_timers.iter_mut() {
        if timer.is_finished() {
            timer.reset();
        }
        timer.timer.tick(time.delta());
    }

    for (ai_transform, mut timer) in player_timers.iter_mut() {
        if timer.is_finished() {
            let near = ai_transform
                .translation
                .distance_squared(player.translation)
                < timer.far_distance * timer.far_distance;
            if near {
                timer.reset_near();
            } else {
                timer.reset_far();
            }
        }
        timer.timer.tick(time.delta());
    }
}
