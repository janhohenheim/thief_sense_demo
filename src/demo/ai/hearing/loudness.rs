use bevy::prelude::*;
use bevy_steam_audio::{
    STEAM_AUDIO_CONTEXT,
    wrapper::{AudionimbusCoordinateSystem, ToSteamAudioVec3},
};
use std::array;

use crate::demo::ai::{
    hearing::{
        AiSource,
        accumulator::AudioInputs,
        debug::AudioDebugWriter,
        node::InputBuffer,
        param::{self, MIN_FRAME_SIZE},
        rms,
    },
    sense::SENSE_INTERVAL_NEAR_TO_FAR,
};

pub(super) fn plugin(app: &mut App) {
    let _ = app;
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
    transform: Query<&GlobalTransform>,
    mut inputs: Query<(AudioInputs, &mut AiSource, &GlobalTransform)>,
    mut writer: Query<&mut AudioDebugWriter>,
    mut buffers: Local<
        Option<(
            [f32; param::MIN_FRAME_SIZE as usize],
            [[f32; param::MIN_FRAME_SIZE as usize]; param::CHANNELS as usize],
            [f32; param::MIN_FRAME_SIZE as usize],
            Vec<f32>,
            audionimbus::DirectEffect,
        )>,
    >,
) -> Result<f32> {
    let listener_transform = AudionimbusCoordinateSystem::from(*transform.get(listener)?);
    let (inputs, mut source, source_transform) = inputs.get_mut(source)?;
    let inputs = inputs.get()?;

    let (direct_buffer, path_buffer, iteration_out_buffer, accumulated_output, direct) = buffers
        .get_or_insert_with(|| {
            (
                [0.0; _],
                [[0.0; _]; _],
                [0.0; _],
                Vec::with_capacity(param::MAX_FRAME_SIZE as usize),
                audionimbus::DirectEffect::try_new(
                    &STEAM_AUDIO_CONTEXT,
                    &param::AUDIO_SETTINGS,
                    &audionimbus::DirectEffectSettings { num_channels: 1 },
                )
                .unwrap(),
            )
        });
    accumulated_output.clear();

    let source_transform = AudionimbusCoordinateSystem::from(*source_transform);

    let channel_ptrs = [direct_buffer.as_mut_ptr()];
    // Safety: all borrowed data is valid until this buffer is dropped again.
    // Also, we pinky promise not to leak the second mutable reference hihi
    let direct_sa_buffer = unsafe {
        audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, MIN_FRAME_SIZE)
    }?;

    let channel_ptrs: [*mut f32; param::CHANNELS as usize] =
        array::from_fn(|i| path_buffer[i].as_mut_ptr());
    // Safety: all borrowed data is valid until this buffer is dropped again.
    // Also, we pinky promise not to leak the second mutable reference hihi
    let path_sa_buffer = unsafe {
        audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, MIN_FRAME_SIZE)
    }?;

    let channel_ptrs = [iteration_out_buffer.as_mut_ptr()];
    // Safety: all borrowed data is valid until this buffer is dropped again.
    // Also, we pinky promise not to leak the second mutable reference hihi
    let mut iteration_out_sa_buffer = unsafe {
        audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, MIN_FRAME_SIZE)
    }?;

    // In 1st order ambisonics, we can just yoink the W channel to get the omnidirectional component
    // Since we only care about the incoming pressure, we don't care about any directionality
    // Source: it was revealed to me in a cryptic dream
    let channel_ptrs = [path_buffer[0].as_mut_ptr()];
    // Safety: all borrowed data is valid until this buffer is dropped again.
    // Also, we pinky promise not to leak the *third* mutable reference hihi
    let omnidir_sa_buffer = unsafe {
        audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, MIN_FRAME_SIZE)
    }?;

    let outputs = source.get_outputs(param::FLAGS);

    let direct_params = audionimbus::DirectEffectParams {
        distance_attenuation: audionimbus::distance_attenuation(
            &STEAM_AUDIO_CONTEXT,
            source_transform.origin.to_steam_audio_vec3(),
            listener_transform.origin.to_steam_audio_vec3(),
            &audionimbus::DistanceAttenuationModel::Default,
        )
        .into(),
        // For some reason I do not understand, the air absorption *hates* our setup:
        // - sometimes causes extremely high values.
        // - generates noise when the source is not occluded even when no input is present
        air_absorption: None,
        ..outputs.direct().into_inner()
    };

    let path_params = audionimbus::PathEffectParams {
        order: param::ORDER,
        binaural: false,
        listener: listener_transform.into(),
        normalize_eq: false,
        ..outputs.pathing().into_inner()
    };

    let repeat = if near { 1 } else { SENSE_INTERVAL_NEAR_TO_FAR };

    // Caching the path creates a shit ton of artifacts on our inputs (maybe related to them being discontinuous due to decimation).
    // I have no clue why. We call reset on it,
    // and I can't see anything in the C++ code that is not cleared. It doesn't need to be wiped per iteration,
    // the below path never produced artifacts for far away NPCs. So it can be cached *to an extent*. I just don't know it.
    // Creating the path seems to be ~N(3 µs, 2 µs) by eyeballing it. Which is not that much, but also shit in the hot path.
    let mut path = audionimbus::PathEffect::try_new(
        &STEAM_AUDIO_CONTEXT,
        &param::AUDIO_SETTINGS,
        &audionimbus::PathEffectSettings {
            max_order: param::ORDER,
            spatialization: None,
        },
    )?;

    for i in 0..repeat {
        let iteration_in =
            &inputs.inputs[i * MIN_FRAME_SIZE as usize..(i + 1) * MIN_FRAME_SIZE as usize];

        // The input we use is highly sporadic given that we reuse whatever window happens do be available from the audio thread.
        // That means we often process the same sound multiple times, with some heavy cuts in it, e.g.
        // - Process sound start
        // - Process sound ongoing
        // - Process sound start
        // - Process sound ongoing
        // - Process sound start
        // - Process sound end
        //
        // Empirically, it seems like the EQ and gain states in these effects are having a lot of trouble with this arrangement.
        // So let's wipe them :) The NPCs won't mind clicks anyways!
        direct.reset();
        path.reset();

        iteration_out_buffer.fill(0.0);

        // Safety: all borrowed data is valid until this buffer is dropped again.
        // Also, we pinky promise not to leak the second mutable reference hihi
        let channel_ptrs = [iteration_in.as_ptr() as *mut f32];
        let in_sa_buffer = unsafe {
            audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, MIN_FRAME_SIZE)
        }?;

        direct.apply(&direct_params, &in_sa_buffer, &direct_sa_buffer);
        iteration_out_sa_buffer.mix(&STEAM_AUDIO_CONTEXT, &direct_sa_buffer);

        path.apply(&path_params, &in_sa_buffer, &path_sa_buffer);
        iteration_out_sa_buffer.mix(&STEAM_AUDIO_CONTEXT, &omnidir_sa_buffer);

        accumulated_output.extend_from_slice(iteration_out_buffer);
    }

    if let Ok(mut writer) = writer.get_mut(listener) {
        // if you want to debug the input, make sure to use the following on near NPCs:
        // `buffer.inputs[..param::MIN_FRAME_SIZE as usize].iter().enumerate()`
        // Otherwise they would debug write the entire 525 ms buffer every 175 ms
        for (i, sample) in accumulated_output.iter().enumerate() {
            writer.write_sample(i, *sample);
        }
    }
    let loudness = rms(accumulated_output);

    Ok(loudness)
}
