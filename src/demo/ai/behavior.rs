use bevy::prelude::*;
use bevy_bae::prelude::*;

use crate::demo::{
    npc::{Npc, movement::TargetEnabled},
    target::Target,
};
pub(super) fn plugin(app: &mut App) {
    app.add_observer(insert_npc_behavior);
}

fn insert_npc_behavior(add: On<Add, Npc>, mut commands: Commands) {
    commands
        .entity(add.entity)
        .insert(npc_behavior())
        .set_prop("patrol", true);
}

pub(crate) fn npc_behavior() -> impl Bundle {
    (
        Plan::new(),
        Select,
        tasks![
            (
                Name::new("Punch player"),
                conditions![
                    Condition::eq("alertness", "high"),
                    Condition::eq("sees_player", true),
                    Condition::eq("in_melee_range", true)
                ],
                Operator::new(punch_player)
            ),
            (
                Name::new("Chase Player"),
                conditions![
                    Condition::eq("alertness", "high"),
                    Condition::eq("sees_player", true)
                ],
                Operator::new(chase_player),
            ),
            (
                conditions![Condition::eq("alertness", "high")],
                Name::new("Search Player"),
                Operator::new(search_player),
            ),
            (
                conditions![Condition::eq("alertness", "moderate")],
                Name::new("Investigate"),
                Operator::new(investigate),
            ),
            (
                // Patrol path corners
                // On conversation: rotate head to conversation partner
                // Else on low alert: rotate head to look around
                conditions![Condition::eq("patrol", true)],
                Name::new("Patrol"),
                Operator::new(patrol),
            ),
            (
                // Fallback idle
                // On conversation: rotate head to conversation partner
                // Else on low alert: rotate head to look around
                Name::new("Idle"),
                Operator::new(idle),
            ),
        ],
    )
}

fn punch_player(_: In<OperatorInput>) -> OperatorStatus {
    info!("Punching player");
    OperatorStatus::Success
}

fn chase_player(_: In<OperatorInput>) -> OperatorStatus {
    info!("Chasing player");
    OperatorStatus::Success
}

fn search_player(_: In<OperatorInput>) -> OperatorStatus {
    info!("Searching player");
    OperatorStatus::Success
}

fn investigate(_: In<OperatorInput>) -> OperatorStatus {
    info!("Investigating");
    OperatorStatus::Success
}

fn patrol(
    input: In<OperatorInput>,
    mut target_enabled: Query<&mut TargetEnabled>,
) -> OperatorStatus {
    let mut target_enabled = target_enabled.get_mut(input.planner).unwrap();
    target_enabled.0 = true;
    OperatorStatus::Success
}

fn idle(_: In<OperatorInput>) -> OperatorStatus {
    OperatorStatus::Success
}
