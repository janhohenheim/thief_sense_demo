use bevy::prelude::*;
use bevy_seedling::sample::{AudioSample, SamplePlayer};
use bevy_shuffle_bag::ShuffleBag;
use rand::rng;

use crate::{asset_tracking::LoadResource as _, demo::npc::animation::NpcStep};

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
                    assets.load("audio/sound_effects/step1.ogg"),
                    assets.load("audio/sound_effects/step2.ogg"),
                    assets.load("audio/sound_effects/step3.ogg"),
                    assets.load("audio/sound_effects/step4.ogg"),
                ],
                &mut rng(),
            )
            .unwrap(),
        }
    }
}

fn play_step_sound(
    step: On<NpcStep>,
    transform: Query<&GlobalTransform>,
    mut commands: Commands,
    mut audio: ResMut<NpcAudio>,
) {
    // It's fine if the transform of the foot is behind by one frame.
    let parent = transform
        .get(step.trigger().animation_player)
        .unwrap()
        .compute_transform();
    let transform = transform.get(step.0).unwrap().compute_transform();
    info!("Step: {}/{}", parent.translation, transform.translation,);
    commands
        .entity(step.0)
        .with_child(SamplePlayer::new(audio.step_sound.pick(&mut rng()).clone()));
}
