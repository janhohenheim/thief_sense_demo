use bevy::prelude::*;
use bevy_seedling::sample::{AudioSample, SamplePlayer};
use bevy_shuffle_bag::ShuffleBag;
use bevy_steam_audio::prelude::*;
use rand::rng;

use crate::{
    animation::AnimationTargetOf,
    asset_tracking::LoadResource as _,
    demo::{ai::hearing::AiAudible, npc::animation::HumanoidStep, player::Player},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(play_step_sound);
    app.load_resource::<NpcAudio>();
}

#[derive(Asset, Resource, Clone, TypePath, Debug)]
struct NpcAudio {
    #[dependency]
    step_sound: ShuffleBag<Handle<AudioSample>>,
}

impl FromWorld for NpcAudio {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            step_sound: ShuffleBag::try_from_iter(
                [
                    assets.load("audio/sound_effects/FootstepsConcrete1.ogg"),
                    assets.load("audio/sound_effects/FootstepsConcrete2.ogg"),
                    assets.load("audio/sound_effects/FootstepsConcrete3.ogg"),
                    assets.load("audio/sound_effects/FootstepsConcrete4.ogg"),
                ],
                &mut rng(),
            )
            .unwrap(),
        }
    }
}

fn play_step_sound(
    step: On<HumanoidStep>,
    mut commands: Commands,
    mut audio: ResMut<NpcAudio>,
    target_of: Query<&AnimationTargetOf>,
    player: Query<(), With<Player>>,
) -> Result {
    let foot = step.trigger().animation_player;
    let root = target_of
        .related(foot)
        .ok_or("Animation target not linked")?;
    let is_player = player.contains(root);
    commands.entity(foot).with_children(|parent| {
        let mut child = parent.spawn((
            Name::new("Footstep Sound"),
            SamplePlayer::new(audio.step_sound.pick(&mut rng()).clone()),
        ));
        if is_player {
            child.insert(AiAudible);
        } else {
            child.insert(SteamAudioPool);
        }
    });
    Ok(())
}
