use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    let _ = app;
}

const SAMPLE_RATE: u32 = 48000;

#[derive(Debug, Resource)]
struct Simulator(audionimbus::Simulator);

impl FromWorld for Simulator {
    fn from_world(world: &mut World) -> Self {
        //let simulator = audionimbus::Simulator::builder();
        //Self(simulator)
        todo!()
    }
}
