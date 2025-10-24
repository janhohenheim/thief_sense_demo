use std::array;

use bevy::prelude::*;
use bevy_steam_audio::{
    STEAM_AUDIO_CONTEXT,
    wrapper::{AudionimbusCoordinateSystem, ToSteamAudioVec3},
};

use crate::demo::ai::hearing::{AiSources, debug::AudioDebugWriter, node::InputBuffer, param, rms};

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
        &mut AiSources,
        &GlobalTransform,
    )>,
    mut writer: Query<&mut AudioDebugWriter>,
) -> Result<f32> {
    let listener_transform = AudionimbusCoordinateSystem::from(*transform.get(listener)?);
    let (mut buffer, mut effects, mut sources, source_transform) = sample_player.get_mut(source)?;
    let SteamAudioEffects {
        near: near_effects,
        far: far_effects,
        path_buffer,
        out_buffer,
    } = effects.as_mut();
    let effects = if near { near_effects } else { far_effects };
    let source = if near {
        &mut sources.near
    } else {
        &mut sources.far
    };
    let source_transform = AudionimbusCoordinateSystem::from(*source_transform);
    let size = if near {
        param::FRAME_SIZE_NEAR
    } else {
        param::FRAME_SIZE_FAR
    } as usize;
    // Safety: all borrowed data is valid until this buffer is dropped again.
    // Also, we pinky promise not to leak the second mutable reference hihi
    let channel_ptrs = [buffer.inputs.as_mut_ptr()];
    let in_buffer =
        unsafe { audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, size as u32) }?;

    for channel in path_buffer.iter_mut() {
        channel.fill(0.0);
    }
    let channel_ptrs: [*mut f32; param::CHANNELS as usize] =
        array::from_fn(|i| path_buffer[i].as_mut_ptr());
    // Safety: all borrowed data is valid until this buffer is dropped again.
    // Also, we pinky promise not to leak the second mutable reference hihi
    let path_out =
        unsafe { audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, size as u32) }?;

    out_buffer.fill(0.0);
    let channel_ptrs = [out_buffer.as_mut_ptr()];
    // Safety: all borrowed data is valid until this buffer is dropped again.
    // Also, we pinky promise not to leak the second mutable reference hihi
    let mut out_sa_buffer =
        unsafe { audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, size as u32) }?;

    let outputs = source.get_outputs(param::FLAGS);

    let now = std::time::Instant::now();
    // TODO:
    // - These effects are per-source, but the same source is active for many NPCs, making the internal state of these effects bogus.
    //   - The effects need to be per-NPC-per-source, not per-source.
    //   - Applying both effects takes about 20 us (rounded up)
    //   - 20 us effect application * 5 sources * 50 NPCS = 5 ms PER FRAME = nope
    //   - Idea:
    //     - only create the npc-source pair when the NPC first heard that source.
    //     - Then keep it alive until the NPC got a netto loudness of 0.0 for like 0.5 seconds in a row (= the state should be wiped now)
    //   - This should dramatically reduce the amount of effects in the world (which, remember, *all* need to be updated every frame for the state to stay valid)
    //   - yes, both direct and pathing are stateful, sorry
    // - Also, when the simulation runs, we only run the far simulation and not the near usually.
    //   - That *may* be fine, just something to keep in mind (source inputs + outputs are outdated for the effect application).
    // - Can we just use one sim?
    //   - It looks to me like the bottleneck in perf is not the simulation, but the effect application, which is *every frame* anyways.
    //   - for comparison, the sim takes 30 us per NPC per update interval
    //     - BUT this may be more in a bigger scene!
    //   - This means at 200 Hz, we need one sim per frame per 12 NPCs (200 / 16.666). At 500 Hz, that's 30 NPCs. much better.
    //   - So it may still be worth it, heck.
    //   - But wait, we don't need to update the sim for any NPC that is not hearing anything
    //     - Then again, we want them all to be very tuned to footsteps, so it will still be plenty NPCs hearing
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
        &out_sa_buffer,
    );

    let mut params = audionimbus::PathEffectParams {
        order: param::ORDER,
        binaural: false,
        listener: listener_transform.into(),
        ..outputs.pathing().into_inner()
    };
    for coeff in &mut params.eq_coeffs {
        *coeff = coeff.max(0.1);
    }

    effects.path.apply(&params, &in_buffer, &path_out);
    info!("Effects took {:?}", now.elapsed());

    // In 1st order ambisonics, we can just yoink the W channel to get the omnidirectional component
    // Since we only care about the incoming pressure, we don't care about any directionality
    // Source: it was revealed to me in a cryptic dream
    let channel_ptrs = [path_buffer[0].as_mut_ptr()];
    // Safety: all borrowed data is valid until this buffer is dropped again.
    // Also, we pinky promise not to leak the *third* mutable reference hihi
    let omnidir_path_out =
        unsafe { audionimbus::AudioBuffer::<&mut [f32], _>::try_new(channel_ptrs, size as u32) }?;

    out_sa_buffer.mix(&STEAM_AUDIO_CONTEXT, &omnidir_path_out);

    if let Ok(mut writer) = writer.get_mut(listener) {
        for (i, sample) in out_buffer[..size].iter().enumerate() {
            writer.write_sample(i, *sample);
        }
    }
    let loudness = rms(&out_buffer[..size]);

    Ok(loudness)
}

fn create_effects(add: On<Add, InputBuffer>, mut commands: Commands) -> Result {
    let near_settings = audionimbus::AudioSettings {
        sampling_rate: param::SAMPLING_RATE,
        frame_size: param::FRAME_SIZE_NEAR,
    };
    let far_settings = audionimbus::AudioSettings {
        sampling_rate: param::SAMPLING_RATE,
        frame_size: param::FRAME_SIZE_FAR,
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
                    max_order: param::ORDER,
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
                    max_order: param::ORDER,
                    spatialization: None,
                },
            )?,
        },
        path_buffer: [[0.0; param::MAX_FRAME_SIZE as usize]; param::CHANNELS as usize],
        out_buffer: [0.0; param::MAX_FRAME_SIZE as usize],
    });
    Ok(())
}

#[derive(Component, Clone, Debug)]
pub(crate) struct SteamAudioEffects {
    near: Effects,
    far: Effects,
    path_buffer: [[f32; param::MAX_FRAME_SIZE as usize]; param::CHANNELS as usize],
    out_buffer: [f32; param::MAX_FRAME_SIZE as usize],
}

#[derive(Clone, Debug)]
struct Effects {
    direct: audionimbus::DirectEffect,
    path: audionimbus::PathEffect,
}
