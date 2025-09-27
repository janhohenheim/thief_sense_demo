use bevy::{
    color::palettes::tailwind, input::keyboard::KeyboardInput,
    picking::pointer::PointerInteraction, prelude::*,
};
use bevy_landmass::{
    AgentDesiredVelocity3d, AgentSettings, AgentTarget3d, Archipelago3d, FromAgentRadius as _,
    PointSampleDistance3d,
};

use crate::{
    demo::player::{PLAYER_RADIUS, PLAYER_RUN_SPEED, PLAYER_WALK_SPEED, Player},
    third_party::landmass::Agent,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, draw_mesh_intersections);
    app.add_observer(move_player);
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

fn move_player(
    mut click: On<Pointer<Click>>,
    mut agents: Query<(&mut AgentSettings, &mut AgentTarget3d)>,
    archipelago: Single<&Archipelago3d>,
    player: Single<&Agent, With<Player>>,
    meshes: Query<(), With<Mesh3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) -> Result {
    let Some(point) = click.hit.position else {
        return Ok(());
    };
    if !meshes.contains(click.original_event_target()) {
        return Ok(());
    }
    let sampled_point = archipelago.sample_point(
        point,
        &PointSampleDistance3d::from_agent_radius(PLAYER_RADIUS),
    );
    let (mut settings, mut target) = agents.get_mut(player.entity())?;
    if let Ok(sampled_point) = sampled_point {
        *target = AgentTarget3d::Point(sampled_point.point());
        settings.desired_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
            PLAYER_RUN_SPEED
        } else {
            PLAYER_WALK_SPEED
        };
        settings.max_speed = settings.desired_speed + 2.0;
        click.propagate(false);
    }
    Ok(())
}
