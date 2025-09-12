use bevy::{color::palettes::tailwind, picking::pointer::PointerInteraction, prelude::*};
use bevy_landmass::{AgentTarget3d, Archipelago3d, FromAgentRadius as _, PointSampleDistance3d};

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
    click: Trigger<Pointer<Click>>,
    mut agents: Query<&mut AgentTarget3d>,
    archipelago: Single<&Archipelago3d>,
) {
    let Some(point) = click.hit.position else {
        return;
    };
    let sampled_point =
        archipelago.sample_point(point, &PointSampleDistance3d::from_agent_radius(5.0));
    if let Ok(sampled_point) = sampled_point {
        for mut target in &mut agents {
            *target = AgentTarget3d::Point(sampled_point.point());
        }
    }
}
