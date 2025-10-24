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
    let channel_ptrs = [buffer.inputs.as_mut_ptr()];
    let in_buffer =
        unsafe { audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, size as u32) }?;

    let mut direct_buffer = [0.0; FRAME_SIZE_FAR as usize];
    let channel_ptrs = [direct_buffer.as_mut_ptr()];
    // Safety: all borrowed data is valid until this buffer is dropped again.
    let direct_out =
        unsafe { audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, size as u32) }?;

    let mut path_buffer = [[0.0; FRAME_SIZE_FAR as usize]; param::CHANNELS as usize];
    let channel_ptrs = [
        path_buffer[0].as_mut_ptr(),
        path_buffer[1].as_mut_ptr(),
        path_buffer[2].as_mut_ptr(),
        path_buffer[3].as_mut_ptr(),
    ];
    // Safety: all borrowed data is valid until this buffer is dropped again.
    let path_out =
        unsafe { audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, size as u32) }?;

    let mut ambisonic_buffer = [0.0; FRAME_SIZE_FAR as usize];
    let channel_ptrs = [ambisonic_buffer.as_mut_ptr()];
    // Safety: all borrowed data is valid until this buffer is dropped again.
    let ambisonic_out =
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

    effects.decode.apply(
        &audionimbus::AmbisonicsDecodeEffectParams {
            order: ORDER,
            hrtf: &audionimbus::Hrtf::from(std::ptr::null_mut()),
            orientation: listener_transform.into(),
            binaural: false,
        },
        &path_out,
        &ambisonic_out,
    );
    if let Ok(mut writer) = writer.get_mut(listener) {
        let output = direct_buffer
            .iter()
            .zip(ambisonic_buffer)
            .map(|(direct, path)| *direct + path);
        for sample in output {
            writer.write_sample(sample).unwrap();
        }
    }
    let loudness_mean_squared = direct_buffer
        .iter()
        .zip(ambisonic_buffer)
        .take(size)
        .fold(0.0, |acc, (direct, path)| {
            acc + (*direct + path) * (*direct + path)
        })
        / size as f32;
    let loudness = loudness_mean_squared.sqrt();

    commands.entity(listener).insert(DebugHearing(loudness));
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
            decode: audionimbus::AmbisonicsDecodeEffect::try_new(
                &STEAM_AUDIO_CONTEXT,
                &near_settings,
                &audionimbus::AmbisonicsDecodeEffectSettings {
                    max_order: ORDER,
                    speaker_layout: audionimbus::SpeakerLayout::Mono,
                    hrtf: &audionimbus::Hrtf::from(std::ptr::null_mut()),
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
            decode: audionimbus::AmbisonicsDecodeEffect::try_new(
                &STEAM_AUDIO_CONTEXT,
                &far_settings,
                &audionimbus::AmbisonicsDecodeEffectSettings {
                    max_order: ORDER,
                    speaker_layout: audionimbus::SpeakerLayout::Mono,
                    hrtf: &audionimbus::Hrtf::from(std::ptr::null_mut()),
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
    decode: audionimbus::AmbisonicsDecodeEffect,
}
