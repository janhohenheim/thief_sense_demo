use bevy::prelude::*;
use bevy_seedling::sample::{AudioSample, SamplePlayer};
use bevy_steam_audio::nodes::SteamAudioPool;
use bevy_trill::Response;

use crate::{
    asset_tracking::LoadResource as _,
    demo::{ai::hearing::node::AiPool, npc::Npc},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(insert_npc_response_observer)
        .add_observer(play_bark)
        .load_asset::<AudioSample>("audio/barks/patrol/patrol_lowest-1.ogg")
        .load_asset::<AudioSample>("audio/barks/patrol/patrol_lowest-2.ogg")
        .load_asset::<AudioSample>("audio/barks/patrol/patrol_lowest-3.ogg")
        .load_asset::<AudioSample>("audio/barks/patrol/patrol_low-1.ogg")
        .load_asset::<AudioSample>("audio/barks/patrol/patrol_low-2.ogg")
        .load_asset::<AudioSample>("audio/barks/patrol/patrol_low-3.ogg")
        .load_asset::<AudioSample>("audio/barks/up_to_low/up_to_low-1.ogg")
        .load_asset::<AudioSample>("audio/barks/up_to_moderate/up_to_moderate-1.ogg")
        .load_asset::<AudioSample>("audio/barks/up_to_high/up_to_high-1.ogg")
        .load_asset::<AudioSample>("audio/barks/down_to_lowest/down_to_lowest-1.ogg")
        .load_asset::<AudioSample>("audio/barks/down_to_lowest/down_to_lowest-2.ogg")
        .load_asset::<AudioSample>("audio/barks/down_to_low/down_to_low-1.ogg")
        .load_asset::<AudioSample>("audio/barks/down_to_moderate/down_to_moderate-1.ogg");
}

fn insert_npc_response_observer(add: On<Add, Npc>, mut commands: Commands) {
    commands.entity(add.entity).observe(on_response);
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

    commands.entity(play.entity).with_children(|parent| {
        let mut bark = parent.spawn((
            SamplePlayer::new(assets.load(play.path.clone())),
            BarkOf {
                entity: play.entity,
                priority: play.priority,
            },
        ));

        if play.priority >= 3 {
            bark.insert(AiPool);
        } else {
            bark.insert(SteamAudioPool);
        }
    });
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
