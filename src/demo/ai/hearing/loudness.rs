use bevy::prelude::*;
use bevy_steam_audio::{
    STEAM_AUDIO_CONTEXT,
    settings::SteamAudioHrtf,
    wrapper::{AudionimbusCoordinateSystem, ToSteamAudioVec3},
};

use crate::demo::ai::{
    debug::DebugHearing,
    hearing::{
        AiHrtfs, AiSources, FRAME_SIZE_FAR, FRAME_SIZE_NEAR, SAMPLING_RATE,
        debug::AudioDebugWriter,
        node::InputBuffer,
        param::{FLAGS, ORDER},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(create_effects);
}

pub(crate) struct LoudnessInput {
    pub(crate) listener: Entity,
    pub(crate) source: Entity,
    pub(crate) near: bool,
}

pub(crate) fn loudness_to_listener(
    In(LoudnessInput {
        listener,
        source,
        near,
    }): In<LoudnessInput>,
    mut commands: Commands,
    transform: Query<&GlobalTransform>,
    mut sample_player: Query<(
        &InputBuffer,
        &mut SteamAudioEffects,
        &mut AiSources,
        &GlobalTransform,
    )>,
    mut writer: Query<&mut AudioDebugWriter>,
    hrtfs: Res<AiHrtfs>,
) -> Result<f32> {
    let listener_transform = AudionimbusCoordinateSystem::from(*transform.get(listener)?);
    let (buffer, mut all_effects, mut sources, source_transform) = sample_player.get_mut(source)?;
    let SteamAudioEffects {
        near: near_effects,
        far: far_effects,
        direct_buffer,
        path_buffer,
    } = all_effects.as_mut();
    let effects = if near { near_effects } else { far_effects };
    let source = if near {
        &mut sources.near
    } else {
        &mut sources.far
    };
    let hrtf = if near { &hrtfs.near } else { &hrtfs.far };
    let source_transform = AudionimbusCoordinateSystem::from(*source_transform);
    let size = if near {
        FRAME_SIZE_NEAR
    } else {
        FRAME_SIZE_FAR
    } as usize;
    let mono_settings = audionimbus::AudioBufferSettings {
        num_channels: Some(1),
        num_samples: Some(size as u32),
        ..default()
    };
    let in_buffer = audionimbus::AudioBuffer::try_with_data_and_settings(
        &buffer.inputs[..size],
        mono_settings,
    )?;
    let direct_out = audionimbus::AudioBuffer::try_with_data_and_settings(
        &mut direct_buffer[..size],
        mono_settings,
    )?;
    let path_out = audionimbus::AudioBuffer::try_with_data_and_settings(
        &mut path_buffer[..size],
        mono_settings,
    )?;

    let outputs = source.get_outputs(FLAGS);

    effects.direct.apply(
        &audionimbus::DirectEffectParams {
            distance_attenuation: audionimbus::distance_attenuation(
                &STEAM_AUDIO_CONTEXT,
                source_transform.origin.to_steam_audio_vec3(),
                listener_transform.origin.to_steam_audio_vec3(),
                &audionimbus::DistanceAttenuationModel::Default,
            )
            .into(),
            air_absorption: audionimbus::air_absorption(
                &STEAM_AUDIO_CONTEXT,
                &source_transform.origin.to_steam_audio_vec3(),
                &listener_transform.origin.to_steam_audio_vec3(),
                &audionimbus::AirAbsorptionModel::Default,
            )
            .into(),
            ..outputs.direct().into_inner()
        },
        &in_buffer,
        &direct_out,
    );

    effects.path.apply(
        &audionimbus::PathEffectParams {
            order: ORDER,
            binaural: true,
            hrtf: hrtf.clone(),
            listener: listener_transform.into(),
            ..outputs.pathing().into_inner()
        },
        &in_buffer,
        &path_out,
    );
    if let Ok(mut writer) = writer.get_mut(listener) {
        let output = direct_buffer
            .iter()
            .zip(path_buffer.iter())
            .map(|(direct, path)| *direct + path);
        for sample in output {
            writer.write_sample(sample).unwrap();
        }
    }
    let loudness_mean_squared = direct_buffer
        .iter()
        .zip(path_buffer)
        .take(size)
        .fold(0.0, |acc, (direct, path)| {
            acc + (*direct + *path) * (*direct + *path)
        })
        / size as f32;
    let loudness = loudness_mean_squared.sqrt();

    commands.entity(listener).insert(DebugHearing(loudness));
    Ok(loudness)
}

fn create_effects(
    add: On<Add, InputBuffer>,
    mut commands: Commands,
    hrtfs: Res<AiHrtfs>,
) -> Result {
    let near_settings = audionimbus::AudioSettings {
        sampling_rate: SAMPLING_RATE,
        frame_size: FRAME_SIZE_NEAR,
    };
    let far_settings = audionimbus::AudioSettings {
        sampling_rate: SAMPLING_RATE,
        frame_size: FRAME_SIZE_FAR,
    };
    let direct_settings = audionimbus::DirectEffectSettings { num_channels: 1 };
    commands.entity(add.entity).try_insert(SteamAudioEffects {
        near: Effects {
            direct: audionimbus::DirectEffect::try_new(
                &STEAM_AUDIO_CONTEXT,
                &near_settings,
                &direct_settings,
            )?,
            path: audionimbus::PathEffect::try_new(
                &STEAM_AUDIO_CONTEXT,
                &near_settings,
                &audionimbus::PathEffectSettings {
                    max_order: ORDER,
                    spatialization: Some(audionimbus::Spatialization {
                        speaker_layout: audionimbus::SpeakerLayout::Mono,
                        hrtf: &hrtfs.near,
                    }),
                },
            )?,
        },
        far: Effects {
            direct: audionimbus::DirectEffect::try_new(
                &STEAM_AUDIO_CONTEXT,
                &far_settings,
                &direct_settings,
            )?,
            path: audionimbus::PathEffect::try_new(
                &STEAM_AUDIO_CONTEXT,
                &far_settings,
                &audionimbus::PathEffectSettings {
                    max_order: ORDER,
                    spatialization: Some(audionimbus::Spatialization {
                        speaker_layout: audionimbus::SpeakerLayout::Mono,
                        hrtf: &hrtfs.far,
                    }),
                },
            )?,
        },
        direct_buffer: vec![0.0; FRAME_SIZE_FAR as usize],
        path_buffer: vec![0.0; FRAME_SIZE_FAR as usize],
    });
    Ok(())
}

#[derive(Component, Clone, Debug)]
pub(crate) struct SteamAudioEffects {
    near: Effects,
    far: Effects,
    direct_buffer: Vec<f32>,
    path_buffer: Vec<f32>,
}

#[derive(Clone, Debug)]
struct Effects {
    direct: audionimbus::DirectEffect,
    path: audionimbus::PathEffect,
}
