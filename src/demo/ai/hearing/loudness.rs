use std::{array, f32::consts::TAU};

use bevy::prelude::*;
use bevy_steam_audio::{
    STEAM_AUDIO_CONTEXT,
    wrapper::{AudionimbusCoordinateSystem, ToSteamAudioVec3},
};

use crate::demo::ai::{
    debug::DebugHearing,
    hearing::{
        AiSources, FRAME_SIZE_FAR, FRAME_SIZE_NEAR, SAMPLING_RATE,
        debug::AudioDebugWriter,
        node::InputBuffer,
        param::{self, FLAGS, ORDER},
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
        &mut InputBuffer,
        &mut SteamAudioEffects,
        &mut AiSources,
        &GlobalTransform,
    )>,
    mut writer: Query<&mut AudioDebugWriter>,
) -> Result<f32> {
    let listener_transform = AudionimbusCoordinateSystem::from(*transform.get(listener)?);
    let (mut buffer, mut all_effects, mut sources, source_transform) =
        sample_player.get_mut(source)?;
    let SteamAudioEffects {
        near: near_effects,
        far: far_effects,
    } = all_effects.as_mut();
    let effects = if near { near_effects } else { far_effects };
    let source = if near {
        &mut sources.near
    } else {
        &mut sources.far
    };
    let source_transform = AudionimbusCoordinateSystem::from(*source_transform);
    let size = if near {
        FRAME_SIZE_NEAR
    } else {
        FRAME_SIZE_FAR
    } as usize;
    // Safety: all borrowed data is valid until this buffer is dropped again.
    // Also, we pinky promise not to leak the second mutable reference hihi
    let channel_ptrs = [buffer.inputs.as_mut_ptr()];
    let in_buffer =
        unsafe { audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, size as u32) }?;

    let mut direct_buffer = [0.0; FRAME_SIZE_FAR as usize];
    let channel_ptrs = [direct_buffer.as_mut_ptr()];
    // Safety: all borrowed data is valid until this buffer is dropped again.
    // Also, we pinky promise not to leak the second mutable reference hihi
    let direct_out =
        unsafe { audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, size as u32) }?;

    let mut path_buffer = [[0.0; FRAME_SIZE_FAR as usize]; param::CHANNELS as usize];
    let channel_ptrs: [*mut f32; param::CHANNELS as usize] =
        array::from_fn(|i| path_buffer[i].as_mut_ptr());
    // Safety: all borrowed data is valid until this buffer is dropped again.
    // Also, we pinky promise not to leak the second mutable reference hihi
    let path_out =
        unsafe { audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, size as u32) }?;

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
            binaural: false,
            listener: listener_transform.into(),
            ..outputs.pathing().into_inner()
        },
        &in_buffer,
        &path_out,
    );

    // In 1st order ambisonics, we can just yoink the W channel to get the omnidirectional component
    // Since we only care about the incoming pressure, we don't care about any directionality
    // The normalization that Steam Audio uses is  0.5 * sqrt(1/pi) = 0.282095
    // Source: it was revealed to me in a cryptic dream
    let mut omnidir_path_component = path_buffer[0];
    for sample in &mut omnidir_path_component {
        // TODO: no clue why that clamp is necessary. Sometimes the pathing generates crazy high values (>1e6).
        // But it sounds alright when just clamping, so uuuuh let's do that for now
        *sample = (*sample / 0.282095).clamp(-1.0, 1.0);
    }

    let mix = direct_buffer
        .iter()
        .copied()
        .zip(omnidir_path_component)
        .take(size)
        .map(|(direct, path)| direct + path);

    if let Ok(mut writer) = writer.get_mut(listener) {
        for sample in mix.clone() {
            writer.write_sample(sample).unwrap();
        }
    }
    let loudness_mean_squared = mix.fold(0.0, |acc, val| acc + val * val) / size as f32;
    let loudness = loudness_mean_squared.sqrt();

    Ok(loudness)
}

fn create_effects(add: On<Add, InputBuffer>, mut commands: Commands) -> Result {
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
                    spatialization: None,
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
                    spatialization: None,
                },
            )?,
        },
    });
    Ok(())
}

#[derive(Component, Clone, Debug)]
pub(crate) struct SteamAudioEffects {
    near: Effects,
    far: Effects,
}

#[derive(Clone, Debug)]
struct Effects {
    direct: audionimbus::DirectEffect,
    path: audionimbus::PathEffect,
}
