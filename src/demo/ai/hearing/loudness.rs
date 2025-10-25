use bevy::prelude::*;
use bevy_steam_audio::{
    STEAM_AUDIO_CONTEXT,
    wrapper::{AudionimbusCoordinateSystem, ToSteamAudioVec3},
};
use std::io::Write;
use std::{array, fs::File, iter};

use crate::demo::ai::{
    hearing::{
        AiSource,
        debug::AudioDebugWriter,
        node::InputBuffer,
        param::{self, MIN_FRAME_SIZE},
        rms,
    },
    sense::SENSE_INTERVAL_NEAR_TO_FAR,
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
    transform: Query<&GlobalTransform>,
    mut sample_player: Query<(
        &mut InputBuffer,
        &mut SteamAudioEffects,
        &mut AiSource,
        &GlobalTransform,
    )>,
    mut writer: Query<&mut AudioDebugWriter>,
) -> Result<f32> {
    let listener_transform = AudionimbusCoordinateSystem::from(*transform.get(listener)?);
    let (mut buffer, mut effects, mut source, source_transform) = sample_player.get_mut(source)?;
    let SteamAudioEffects {
        direct,
        path,
        direct_buffer,
        path_buffer,
        iteration_out_buffer,
        accumulated_output,
    } = effects.as_mut();
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

    let path_params = {
        let mut params = audionimbus::PathEffectParams {
            order: param::ORDER,
            binaural: false,
            listener: listener_transform.into(),
            ..outputs.pathing().into_inner()
        };
        for coeff in &mut params.eq_coeffs {
            *coeff = coeff.max(0.1);
        }
        params
    };

    let mut coeffs = [0.0; 20];
    coeffs[0] = 1.0;
    let mut path_params = audionimbus::PathEffectParams {
        //eq_coeffs: [0.5; 3],
        sh_coeffs: audionimbus::ShCoeffs(coeffs.as_mut_ptr()),
        order: param::ORDER,
        binaural: false,
        hrtf: audionimbus::Hrtf::from(std::ptr::null_mut()),
        listener: listener_transform.into(),
        normalize_eq: false,
        ..outputs.pathing().into_inner()
    };
    for coeff in &mut path_params.eq_coeffs {
        info!(?coeff);
        *coeff = coeff.max(0.1);
    }

    let repeat = if near { 1 } else { SENSE_INTERVAL_NEAR_TO_FAR };
    let now = std::time::Instant::now();
    for i in 0..repeat {
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

        let iteration_in =
            &mut buffer.inputs[i * MIN_FRAME_SIZE as usize..(i + 1) * MIN_FRAME_SIZE as usize];

        // Safety: all borrowed data is valid until this buffer is dropped again.
        // Also, we pinky promise not to leak the second mutable reference hihi
        let channel_ptrs = [iteration_in.as_mut_ptr()];
        let in_sa_buffer = unsafe {
            audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, MIN_FRAME_SIZE)
        }?;

        // TODO:
        // - These effects are per-source, but the same source is active for many NPCs, making the internal state of these effects bogus.
        //   - The effects need to be per-NPC-per-source, not per-source.

        direct.apply(&direct_params, &in_sa_buffer, &direct_sa_buffer);
        // iteration_out_sa_buffer.mix(&STEAM_AUDIO_CONTEXT, &direct_sa_buffer);

        path.apply(&path_params, &in_sa_buffer, &path_sa_buffer);
        iteration_out_sa_buffer.mix(&STEAM_AUDIO_CONTEXT, &omnidir_sa_buffer);

        accumulated_output.extend_from_slice(iteration_out_buffer);
    }
    debug!("Effects took {:?}", now.elapsed());

    if let Ok(mut writer) = writer.get_mut(listener) {
        for (i, sample) in accumulated_output.iter().enumerate() {
            writer.write_sample(i, *sample);
        }
    }
    let loudness = rms(accumulated_output);
    if loudness > 1.0 {
        let mut file = File::create("debug/data.csv").unwrap();
        for sample in accumulated_output.iter() {
            write!(file, "{}\n", sample).unwrap();
        }
        panic!("loudness: {loudness}");
    }

    Ok(loudness)
}

fn detect_hampel(signal: &[f32], half_win: usize, thresh: f32) -> Vec<bool> {
    let n = signal.len();
    let mut is_spike = vec![false; n];
    if n == 0 {
        return is_spike;
    }

    // For each center sample, compute median and MAD in window [i-half_win .. i+half_win]
    for i in 0..n {
        let lo = i.saturating_sub(half_win);
        let hi = (i + half_win).min(n - 1);
        let mut win: Vec<f32> = signal[lo..=hi].to_vec();
        let med = median(&mut win);
        // compute absolute deviations
        for v in win.iter_mut() {
            *v = (*v - med).abs();
        }
        let mad = median(&mut win);
        let sigma_est = 1.4826 * (if mad == 0.0 { 1e-12 } else { mad }); // avoid zero
        let z = ((signal[i] - med).abs()) / sigma_est;
        if z > thresh {
            is_spike[i] = true;
        }
    }
    is_spike
}

fn remove_spikes_interp(signal: &mut [f32], is_spike: &[bool]) {
    let n = signal.len();
    let mut i = 0usize;
    while i < n {
        if !is_spike[i] {
            i += 1;
            continue;
        }
        // start of spike run
        let start = i;
        while i < n && is_spike[i] {
            i += 1;
        }
        let end = i; // [start, end) are spikes

        // find left neighbor (start-1) and right neighbor (end)
        let left_idx = if start == 0 { None } else { Some(start - 1) };
        let right_idx = if end >= n { None } else { Some(end) };

        match (left_idx, right_idx) {
            (Some(l), Some(r)) => {
                let left = signal[l];
                let right = signal[r];
                let len = (end - start) as f32 + 1.0;
                for k in 0..(end - start) {
                    let alpha = (k as f32 + 1.0) / (len + 1.0); // fraction between left and right
                    signal[start + k] = left * (1.0 - alpha) + right * alpha;
                }
            }
            (Some(l), None) => {
                // trailing spikes: fill with left neighbor value (or repeat)
                for k in start..end {
                    signal[k] = signal[l];
                }
            }
            (None, Some(r)) => {
                // leading spikes: fill with right neighbor
                for k in start..end {
                    signal[k] = signal[r];
                }
            }
            (None, None) => { /* all signal is spikes: do nothing */ }
        }
    }
}

fn median(slice: &mut [f32]) -> f32 {
    slice.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = slice.len();
    if n % 2 == 1 {
        slice[n / 2]
    } else {
        0.5 * (slice[n / 2 - 1] + slice[n / 2])
    }
}

fn create_effects(add: On<Add, InputBuffer>, mut commands: Commands) -> Result {
    let settings = audionimbus::AudioSettings {
        sampling_rate: param::SAMPLING_RATE,
        frame_size: param::MIN_FRAME_SIZE,
    };
    commands.entity(add.entity).try_insert(SteamAudioEffects {
        direct: audionimbus::DirectEffect::try_new(
            &STEAM_AUDIO_CONTEXT,
            &settings,
            &audionimbus::DirectEffectSettings { num_channels: 1 },
        )?,
        path: audionimbus::PathEffect::try_new(
            &STEAM_AUDIO_CONTEXT,
            &settings,
            &audionimbus::PathEffectSettings {
                max_order: param::ORDER,
                spatialization: None,
            },
        )?,
        direct_buffer: [0.0; param::MIN_FRAME_SIZE as usize],
        path_buffer: [[0.0; param::MIN_FRAME_SIZE as usize]; param::CHANNELS as usize],
        iteration_out_buffer: [0.0; param::MIN_FRAME_SIZE as usize],
        accumulated_output: Vec::with_capacity(param::MAX_FRAME_SIZE as usize),
    });
    Ok(())
}

#[derive(Component, Clone, Debug)]
pub(crate) struct SteamAudioEffects {
    direct: audionimbus::DirectEffect,
    path: audionimbus::PathEffect,
    direct_buffer: [f32; param::MIN_FRAME_SIZE as usize],
    path_buffer: [[f32; param::MIN_FRAME_SIZE as usize]; param::CHANNELS as usize],
    iteration_out_buffer: [f32; param::MIN_FRAME_SIZE as usize],
    accumulated_output: Vec<f32>,
}
