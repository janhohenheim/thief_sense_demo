use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use bevy_steam_audio::{
    STEAM_AUDIO_CONTEXT,
    simulation::{AudionimbusSimulator, SteamAudioReady},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Simulator>();
    app.add_systems(Startup, kick_off_simulation);
}

/// Nyquist frequency is 4k, that's two octaves above 1k, which the human ear is most sensitive to.
const SAMPLING_RATE: u32 = 8000;

/// in seconds
const HEARING_INTERVAL: f32 = 0.2;

const FRAME_SIZE: u32 = ((SAMPLING_RATE as f32) * HEARING_INTERVAL) as u32;

#[derive(Debug, Resource, Deref, DerefMut)]
struct Simulator(
    Arc<RwLock<audionimbus::Simulator<audionimbus::Direct, audionimbus::Reflections>>>,
);

#[derive(Resource)]
struct AsyncSimulationSynchronization {
    sender: crossbeam_channel::Sender<()>,
    complete: Arc<AtomicBool>,
}

impl FromWorld for Simulator {
    fn from_world(_world: &mut World) -> Self {
        let simulator = audionimbus::Simulator::builder(
            audionimbus::SceneParams::Default,
            SAMPLING_RATE,
            FRAME_SIZE,
        )
        .with_direct(audionimbus::DirectSimulationSettings {
            // We use raycasts, not volumetric
            max_num_occlusion_samples: 0,
        })
        .with_reflections(audionimbus::ReflectionsSimulationSettings::Convolution {
            max_num_rays: 1024,
            num_diffuse_samples: 1024,
            max_duration: 1.0,
            max_order: 1,
            max_num_sources: 200,
            num_threads: 1,
        })
        .try_build(&STEAM_AUDIO_CONTEXT)
        .unwrap();
        Self(Arc::new(RwLock::new(simulator)))
    }
}

fn kick_off_simulation(mut commands: Commands, simulator: Res<Simulator>) {
    let simulation_complete = Arc::new(AtomicBool::new(false));
    let simulation_complete_inner = simulation_complete.clone();
    let (tx, rx) = crossbeam_channel::unbounded::<()>();
    commands.insert_resource(AsyncSimulationSynchronization {
        sender: tx,
        complete: simulation_complete,
    });

    let simulator = simulator.clone();
    let future = async move {
        loop {
            {
                // Block thread until simulator is ready
                let simulator = simulator.read().unwrap();
                simulator.run_reflections();
            }

            simulation_complete_inner.store(true, Ordering::Relaxed);
            if rx.recv().is_err() {
                // tx dropped because we created a new simulation
                break;
            }
        }
    };
    AsyncComputeTaskPool::get().spawn(future).detach();
}

fn sync_simulators(
    _ready: On<SteamAudioReady>,
    npc_simulator: ResMut<Simulator>,
    main_simulator: Res<AudionimbusSimulator>,
) {
}
