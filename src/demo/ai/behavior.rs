use bevy::prelude::*;
use bevy_bae::prelude::*;
use bevy_seedling::sample::{AudioSample, SamplePlayer};
use bevy_steam_audio::nodes::SteamAudioPool;
use bevy_trill::{RequestResponse, Response};

use crate::{
    asset_tracking::LoadResource,
    demo::{
        ai::{
            alertness::{Alertness, ChangeAlertness},
            awareness::{Awareness, AwarenessLevel},
        },
        npc::{Npc, movement::TargetEnabled},
    },
    log_normal_timer::LogNormalTimer,
};
use bevy_bae::bevy_mod_props::Class;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(insert_npc_behavior)
        .add_observer(sync_alertness)
        .add_observer(play_bark)
        .load_asset::<AudioSample>("audio/barks/patrol/patrol-1.ogg")
        .load_asset::<AudioSample>("audio/barks/patrol/patrol-2.ogg")
        .load_asset::<AudioSample>("audio/barks/patrol/patrol-3.ogg")
        .load_asset::<AudioSample>("audio/barks/up_to_low/up_to_low-1.ogg")
        .load_asset::<AudioSample>("audio/barks/up_to_moderate/up_to_moderate-1.ogg")
        .load_asset::<AudioSample>("audio/barks/up_to_high/up_to_high-1.ogg");
}

fn insert_npc_behavior(add: On<Add, Npc>, mut commands: Commands) {
    commands
        .entity(add.entity)
        .insert(npc_behavior())
        .set_prop("patrol", true)
        .observe(on_response);
}

pub(crate) fn npc_behavior() -> impl Bundle {
    (
        Plan::new(),
        Select,
        Class::new("guard"),
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

fn on_response(response: On<Response>, mut commands: Commands) {
    let npc = response.event_target();
    let Some(line) = response.get("line") else {
        return;
    };
    let Some(priority) = response.get("priority") else {
        return;
    };
    commands.trigger(PlayBark {
        entity: npc,
        path: line.to_string(),
        priority: priority.parse().unwrap(),
    });
}

fn sync_alertness(
    change: On<ChangeAlertness>,
    mut npcs: Query<(&Alertness, &mut Props)>,
    mut trill_writer: MessageWriter<RequestResponse>,
) -> Result {
    let (alertness, mut props) = npcs.get_mut(change.npc)?;
    let old = change.previous_level;
    let new = alertness.level;
    props.set("alertness", new.to_string());
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

#[derive(EntityEvent)]
struct PlayBark {
    entity: Entity,
    path: String,
    priority: i32,
}

fn play_bark(
    play: On<PlayBark>,
    npcs: Query<&Bark>,
    barks: Query<&BarkOf>,
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    let is_allowed_to_bark = if let Ok(bark) = npcs.get(play.entity)
        && let Ok(bark) = barks.get(bark.0)
    {
        bark.priority < play.priority
    } else {
        true
    };

    if !is_allowed_to_bark {
        return;
    }
    if let Ok(bark) = npcs.get(play.entity) {
        commands.entity(bark.0).try_despawn();
    }

    commands.entity(play.entity).with_child((
        SamplePlayer::new(assets.load(play.path.clone())),
        SteamAudioPool,
        BarkOf {
            entity: play.entity,
            priority: play.priority,
        },
    ));
}

#[derive(Component)]
#[relationship(relationship_target=Bark)]
struct BarkOf {
    #[relationship]
    entity: Entity,
    priority: i32,
}

#[derive(Component)]
#[relationship_target(relationship=BarkOf)]
struct Bark(Entity);
