use bevy::prelude::*;
use bevy_bae::prelude::*;
use bevy_landmass::{
    AgentTarget, AgentTarget3d, Archipelago3d, FromAgentRadius, PointSampleDistance3d,
};
use bevy_trill::RequestResponse;

use crate::{
    demo::{
        ai::{
            alertness::{Alertness, ChangeAlertness},
            awareness::{AwarenessLevel, AwarenessQuery},
        },
        npc::{Npc, movement::TargetEnabled},
    },
    log_normal_timer::LogNormalTimer,
    third_party::landmass::Agent,
};
use bevy_bae::bevy_mod_props::Class;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(insert_npc_behavior)
        .add_observer(sync_alertness);
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
        Class::new("guard"),
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

fn chase_player(
    input: In<OperatorInput>,
    npcs: Query<(&Alertness, &Agent)>,
    mut agents: Query<&mut AgentTarget3d>,
    awareness: AwarenessQuery,
    mut timer: Local<Option<LogNormalTimer>>,
    time: Res<Time>,
    mut investigation_pos: Local<Option<Vec3>>,
    archipelago: Single<&Archipelago3d>,
) -> OperatorStatus {
    let (alertness, agent) = npcs.get(input.entity).unwrap();
    let last_object = alertness.last_awareness_object.unwrap();
    let awareness = awareness.get(input.entity, last_object).unwrap();

    let timer = timer.get_or_insert_with(|| LogNormalTimer::new(0.1, 0.1));
    timer.tick(time.delta());

    let pos = if timer.is_finished() {
        timer.reset();
        archipelago
            .sample_point(
                awareness.last_pos,
                &PointSampleDistance3d::from_agent_radius(0.5),
            )
            .map(|p| p.point())
    } else if investigation_pos.is_none() {
        archipelago
            .sample_point(
                awareness.last_pos,
                &PointSampleDistance3d::from_agent_radius(0.5),
            )
            .map(|p| p.point())
    } else {
        Ok(investigation_pos.unwrap())
    };
    let Ok(pos) = pos else {
        return OperatorStatus::Failure;
    };
    investigation_pos.replace(pos);
    let mut agent_target = agents.get_mut(agent.get()).unwrap();
    *agent_target = AgentTarget::Point(pos);

    OperatorStatus::Success
}

fn search_player(
    input: In<OperatorInput>,
    npcs: Query<(&Alertness, &Agent)>,
    mut agents: Query<&mut AgentTarget3d>,
    awareness: AwarenessQuery,
    mut timer: Local<Option<LogNormalTimer>>,
    time: Res<Time>,
    mut investigation_pos: Local<Option<Vec3>>,
    archipelago: Single<&Archipelago3d>,
) -> OperatorStatus {
    let (alertness, agent) = npcs.get(input.entity).unwrap();
    let last_object = alertness.last_awareness_object.unwrap();
    let awareness = awareness.get(input.entity, last_object).unwrap();

    let timer = timer.get_or_insert_with(|| LogNormalTimer::new(0.5, 0.1));
    timer.tick(time.delta());

    let pos = if timer.is_finished() {
        timer.reset();
        archipelago
            .sample_point(
                awareness.last_pos,
                &PointSampleDistance3d::from_agent_radius(1.0),
            )
            .map(|p| p.point())
    } else if investigation_pos.is_none() {
        archipelago
            .sample_point(
                awareness.last_pos,
                &PointSampleDistance3d::from_agent_radius(1.0),
            )
            .map(|p| p.point())
    } else {
        Ok(investigation_pos.unwrap())
    };
    let Ok(pos) = pos else {
        return OperatorStatus::Failure;
    };
    investigation_pos.replace(pos);
    let mut agent_target = agents.get_mut(agent.get()).unwrap();
    *agent_target = AgentTarget::Point(pos);

    OperatorStatus::Success
}

fn investigate(
    input: In<OperatorInput>,
    npcs: Query<(&Alertness, &Agent)>,
    mut agents: Query<&mut AgentTarget3d>,
    awareness: AwarenessQuery,
    mut timer: Local<Option<LogNormalTimer>>,
    time: Res<Time>,
    mut investigation_pos: Local<Option<Vec3>>,
    archipelago: Single<&Archipelago3d>,
) -> OperatorStatus {
    let (alertness, agent) = npcs.get(input.entity).unwrap();
    let last_object = alertness.last_awareness_object.unwrap();
    let awareness = awareness.get(input.entity, last_object).unwrap();

    let timer = timer.get_or_insert_with(|| LogNormalTimer::new(4.0, 0.2));
    timer.tick(time.delta());

    let pos = if timer.is_finished() {
        timer.reset();
        archipelago
            .sample_point(
                awareness.last_pos,
                &PointSampleDistance3d::from_agent_radius(3.0),
            )
            .map(|p| p.point())
    } else if investigation_pos.is_none() {
        archipelago
            .sample_point(
                awareness.last_pos,
                &PointSampleDistance3d::from_agent_radius(3.0),
            )
            .map(|p| p.point())
    } else {
        Ok(investigation_pos.unwrap())
    };
    let Ok(pos) = pos else {
        return OperatorStatus::Failure;
    };
    investigation_pos.replace(pos);
    let mut agent_target = agents.get_mut(agent.get()).unwrap();
    *agent_target = AgentTarget::Point(pos);

    OperatorStatus::Success
}

fn patrol(
    input: In<OperatorInput>,
    mut npc: Query<&mut TargetEnabled>,
    mut timer: Local<Option<LogNormalTimer>>,
    time: Res<Time>,
    mut trill_writer: MessageWriter<RequestResponse>,
) -> OperatorStatus {
    let mut target_enabled = npc.get_mut(input.entity).unwrap();
    target_enabled.0 = true;
    let timer = timer.get_or_insert_with(|| LogNormalTimer::new(7.0, 0.2));
    timer.tick(time.delta());
    if timer.is_finished() {
        trill_writer.write(RequestResponse::new(input.entity, "patrol"));
        timer.reset();
    }
    OperatorStatus::Success
}

fn idle(_: In<OperatorInput>) -> OperatorStatus {
    OperatorStatus::Success
}

fn sync_alertness(
    change: On<ChangeAlertness>,
    mut npcs: Query<(&Alertness, &mut Props)>,
    mut trill_writer: MessageWriter<RequestResponse>,
) -> Result {
    let (alertness, mut props) = npcs.get_mut(change.npc)?;
    let old = change.previous_level;
    let new = alertness.level;
    props.set("alertness", new.to_string().to_lowercase());
    props.set(
        "ever_got_high_alerted_by_player",
        alertness.ever_got_high_alerted_by_player,
    );
    if old < new {
        match new {
            AwarenessLevel::Low => {
                trill_writer.write(RequestResponse::new(change.npc, "up_to_low"));
            }
            AwarenessLevel::Moderate => {
                trill_writer.write(RequestResponse::new(change.npc, "up_to_moderate"));
            }
            AwarenessLevel::High => {
                trill_writer.write(RequestResponse::new(change.npc, "up_to_high"));
            }
            _ => {}
        }
    }
    if old > new {
        match new {
            AwarenessLevel::Lowest => {
                trill_writer.write(RequestResponse::new(change.npc, "down_to_lowest"));
            }
            AwarenessLevel::Low => {
                trill_writer.write(RequestResponse::new(change.npc, "down_to_low"));
            }
            AwarenessLevel::Moderate => {
                trill_writer.write(RequestResponse::new(change.npc, "down_to_moderate"));
            }
            _ => {}
        }
    }

    Ok(())
}
