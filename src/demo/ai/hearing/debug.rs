use std::{
    fs::{self, File},
    path::PathBuf,
    sync::Mutex,
};

use bevy::{color::palettes::tailwind, prelude::*};
use hound::{SampleFormat, WavSpec, WavWriter};

use crate::demo::{ai::hearing::param, npc::Npc};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<PathVisualizations>()
        .init_resource::<EnableAudioPathVisualization>()
        .init_resource::<EnableAudioWriter>();
    app.add_systems(PostUpdate, tick_visualizations);
    app.add_observer(add_writer);
}

fn tick_visualizations(
    visualizations: ResMut<PathVisualizations>,
    enabled: Res<EnableAudioPathVisualization>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let mut visualizations = visualizations.lock().unwrap();
    visualizations.retain_mut(|vis| {
        if !enabled.0 {
            return false;
        }
        gizmos.line(
            vis.from,
            vis.to,
            if vis.occluded {
                tailwind::RED_700.with_alpha(0.3)
            } else {
                tailwind::GREEN_700.with_alpha(0.3)
            },
        );
        vis.timer.tick(time.delta());
        !vis.timer.is_finished()
    });
}

#[derive(Resource, Default, Clone, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub(crate) struct EnableAudioWriter(pub(crate) bool);

#[derive(Resource, Default, Clone, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub(crate) struct EnableAudioPathVisualization(pub(crate) bool);

pub(crate) type VisualizationsPtr = *mut Mutex<Vec<PathVisualization>>;

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct PathVisualizations(pub(crate) Mutex<Vec<PathVisualization>>);

#[derive(Reflect, Debug)]
pub(crate) struct PathVisualization {
    pub(crate) from: Vec3,
    pub(crate) to: Vec3,
    pub(crate) occluded: bool,
    timer: Timer,
}

impl PathVisualization {
    pub fn new(
        from: audionimbus_sys::IPLVector3,
        to: audionimbus_sys::IPLVector3,
        occluded: audionimbus_sys::IPLbool,
    ) -> Self {
        Self {
            from: vec3(from.x, from.y, from.z),
            to: vec3(to.x, to.y, to.z),
            occluded: occluded as u32 != 0,
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct AudioDebugWriter(WavWriter<File>);

fn add_writer(add: On<Add, Npc>, name: Query<NameOrEntity>, mut commands: Commands) -> Result {
    let name = name.get(add.entity).unwrap();
    let debug_dir = PathBuf::from("debug");
    fs::create_dir_all(&debug_dir).unwrap();
    let debug_file = debug_dir.join(format!("{name}.wav"));
    let debug_file = File::create(debug_file)?;
    let writer = AudioDebugWriter(WavWriter::new(
        debug_file,
        WavSpec {
            channels: 1,
            sample_rate: param::SAMPLING_RATE,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        },
    )?);
    commands.entity(add.entity).insert(writer);
    Ok(())
}
