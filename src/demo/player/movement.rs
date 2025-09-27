use bevy::{color::palettes::tailwind, picking::pointer::PointerInteraction, prelude::*};
use bevy_landmass::{AgentTarget3d, Archipelago3d, FromAgentRadius as _, PointSampleDistance3d};

use crate::{demo::player::Player, third_party::landmass::Agent};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, draw_mesh_intersections);
    app.add_observer(on_click);
}

/// A system that draws hit indicators for every pointer.
fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for point in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position)
    {
        gizmos.sphere(point, 0.1, tailwind::RED_500);
    }
}

fn on_click(
    click: On<Pointer<Click>>,
    mut agent_targets: Query<&mut AgentTarget3d>,
    archipelago: Single<&Archipelago3d>,
    player_agent: Single<&Agent, With<Player>>,
    meshes: Query<(), With<Mesh3d>>,
) -> Result {
    let Some(point) = click.hit.position else {
        return Ok(());
    };
    if !meshes.contains(click.original_event_target()) {
        return Ok(());
    }
    let sampled_point =
        archipelago.sample_point(point, &PointSampleDistance3d::from_agent_radius(5.0));
    let mut agent_target = agent_targets.get_mut(player_agent.entity())?;
    if let Ok(sampled_point) = sampled_point {
        *agent_target = AgentTarget3d::Point(sampled_point.point());
    }
    Ok(())
}
