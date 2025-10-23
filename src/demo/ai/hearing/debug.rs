use std::sync::{Arc, Mutex};

use bevy::{color::palettes::tailwind, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<PathVisualizations>()
        .init_resource::<EnableAudioPathVisualization>();
    app.add_systems(PostUpdate, tick_visualizations);
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
pub(crate) struct EnableAudioPathVisualization(pub(crate) bool);

pub(crate) type VisualizationsPtr = *mut Arc<Mutex<Vec<PathVisualization>>>;

#[derive(Resource, Default, Clone, Deref, DerefMut)]
pub(crate) struct PathVisualizations(pub(crate) Box<Arc<Mutex<Vec<PathVisualization>>>>);

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
