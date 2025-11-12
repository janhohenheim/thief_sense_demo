use bevy::prelude::*;
use bevy_seedling::sample::{AudioSample, SamplePlayer};
use bevy_shuffle_bag::ShuffleBag;
use bevy_steam_audio::prelude::*;
use bevy_trenchbroom::geometry::MapGeometry;
use rand::rng;

use crate::{
    animation::AnimationTargetOf,
    asset_tracking::LoadResource as _,
    demo::{ai::hearing::node::AiPool, npc::animation::HumanoidStep, player::Player},
    movement::IsRunning,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(play_step_sound);
    app.load_resource::<NpcAudio>();
}

#[derive(Asset, Resource, Clone, TypePath, Debug)]
struct NpcAudio {
    #[dependency]
    rock_walk: ShuffleBag<Handle<AudioSample>>,
    #[dependency]
    rock_run: ShuffleBag<Handle<AudioSample>>,
    #[dependency]
    gravel_walk: ShuffleBag<Handle<AudioSample>>,
    #[dependency]
    gravel_run: ShuffleBag<Handle<AudioSample>>,
    #[dependency]
    metal_walk: ShuffleBag<Handle<AudioSample>>,
    #[dependency]
    metal_run: ShuffleBag<Handle<AudioSample>>,
}

impl FromWorld for NpcAudio {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            rock_walk: ShuffleBag::try_from_iter(
                ["01", "02", "03", "04", "05", "06", "07", "08", "09"].map(|n| {
                    assets.load(format!(
                        "audio/sound_effects/footsteps/rock/walk/Footsteps_Rock_Walk_{n}.ogg"
                    ))
                }),
                &mut rng(),
            )
            .unwrap(),
            rock_run: ShuffleBag::try_from_iter(
                ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10"].map(|n| {
                    assets.load(format!(
                        "audio/sound_effects/footsteps/rock/run/Footsteps_Rock_Run_{n}.ogg"
                    ))
                }),
                &mut rng(),
            )
            .unwrap(),
            gravel_walk: ShuffleBag::try_from_iter(
                ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10"].map(|n| {
                    assets.load(format!(
                        "audio/sound_effects/footsteps/gravel/walk/Footsteps_Gravel_Walk_{n}.ogg"
                    ))
                }),
                &mut rng(),
            )
            .unwrap(),
            gravel_run: ShuffleBag::try_from_iter(
                ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10"].map(|n| {
                    assets.load(format!(
                        "audio/sound_effects/footsteps/gravel/run/Footsteps_Gravel_Run_{n}.ogg"
                    ))
                }),
                &mut rng(),
            )
            .unwrap(),
            metal_walk: ShuffleBag::try_from_iter(
                [
                    "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13",
                    "14", "15",
                ]
                .map(|n| {
                    assets.load(format!(
                        "audio/sound_effects/footsteps/metal/walk/Footsteps_MetalV1_Walk_{n}.ogg"
                    ))
                }),
                &mut rng(),
            )
            .unwrap(),
            metal_run: ShuffleBag::try_from_iter(
                [
                    "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13",
                    "14", "15",
                ]
                .map(|n| {
                    assets.load(format!(
                        "audio/sound_effects/footsteps/metal/run/Footsteps_MetalV1_Run_{n}.ogg"
                    ))
                }),
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
    roots: Query<(Has<Player>, &IsRunning)>,
    transform: Query<&GlobalTransform>,
    names: Query<&Name>,
    map_geometry: Query<(), With<MapGeometry>>,
    mut ray_cast: MeshRayCast,
) -> Result {
    let foot = step.trigger().animation_player;
    let root = target_of
        .related(foot)
        .ok_or("Animation target not linked")?;
    let (is_player, is_running) = roots.get(root)?;

    let root_transform = transform.get(root)?;
    let hit = ray_cast.cast_ray(
        Ray3d::new(root_transform.translation(), Dir3::NEG_Y),
        &MeshRayCastSettings::default().with_filter(&|entity| map_geometry.contains(entity)),
    );
    let (hit, _hit_data) = hit.first().ok_or("Footstep happened in thin air")?;
    let hit_name = names.get(*hit)?;
    let color = hit_name
        .split('/')
        .next()
        .unwrap_or_default()
        .to_lowercase();
    let bag = match (color.as_ref(), is_running.0) {
        ("green", false) => &mut audio.rock_walk,
        ("green", true) => &mut audio.rock_run,
        ("red", false) => &mut audio.metal_walk,
        ("red", true) => &mut audio.metal_run,
        ("purple", false) => &mut audio.gravel_walk,
        ("purple", true) => &mut audio.gravel_run,
        (_, false) => &mut audio.rock_walk,
        (_, true) => &mut audio.rock_run,
    };
    commands.entity(foot).with_children(|parent| {
        let mut child = parent.spawn((
            Name::new("Footstep Sound"),
            SamplePlayer::new(bag.pick(&mut rng()).clone()),
        ));
        if is_player {
            child.insert(AiPool);
        } else {
            child.insert(SteamAudioPool);
        }
    });
    Ok(())
}
