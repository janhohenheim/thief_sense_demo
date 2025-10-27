use bevy::prelude::*;

pub(crate) mod awareness;
pub(crate) mod debug;
pub(crate) mod hearing;
pub(crate) mod sense;
pub(crate) mod vision;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        sense::plugin,
        awareness::plugin,
        vision::plugin,
        hearing::plugin,
        debug::plugin,
    ));
}

/// f32[0, 1] -> u8[0, 100]
fn calc_control_rating(fraction: f32, low: u8, mid: u8, high: u8) -> u8 {
    let raw = (fraction * 100.0).clamp(1.0, 100.0) as u8;
    const LOW_NORM: u8 = 25;
    const MID_NORM: u8 = 50;
    const HIGH_NORM: u8 = 75;
    let (pre_norm_base, pre_norm_range, norm_base, norm_range) = match raw {
        l if l < low => (0, low, 0, LOW_NORM),
        l if l < mid => (low, mid - low, LOW_NORM, MID_NORM - LOW_NORM),
        l if l < high => (mid, high - mid, MID_NORM, HIGH_NORM - MID_NORM),
        _ => (high, 100 - high, HIGH_NORM, 100 - HIGH_NORM),
    };

    norm_base + ((raw - pre_norm_base) as f32 / pre_norm_range as f32) as u8 + norm_range
}
