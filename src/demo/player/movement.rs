use bevy::{picking::pointer::PointerInteraction, prelude::*};
use bevy_landmass::{
    AgentSettings, AgentTarget3d, Archipelago3d, FromAgentRadius as _, PointSampleDistance3d,
};

use crate::{
    cpu_lighting::{estimate_point_light, estimate_spot_light, estimate_tone_mapping},
    demo::player::{PLAYER_RADIUS, PLAYER_RUN_SPEED, PLAYER_WALK_SPEED, Player},
    third_party::landmass::Agent,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, draw_cursor_intersections);
    app.add_observer(move_player);
}

/// A system that draws hit indicators for every pointer.
fn draw_cursor_intersections(
    pointers: Query<&PointerInteraction>,
    mut gizmos: Gizmos,
    point_lights: Query<(&GlobalTransform, &PointLight)>,
    spot_lights: Query<(&GlobalTransform, &SpotLight)>,
    directional_lights: Query<&DirectionalLight>,
) {
    for point in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position)
    {
        let mut intensity = 0.0;
        for (light_transform, light) in &point_lights {
            intensity += estimate_point_light(*light, light_transform.translation(), point);
        }
        for (light_transform, light) in &spot_lights {
            intensity += estimate_spot_light(*light, light_transform.compute_transform(), point);
        }
        for _light in &directional_lights {
            //intensity += estimate_directional_light(light.clone());
        }
        let brightness = estimate_tone_mapping(intensity);

        gizmos.sphere(
            point,
            0.5,
            (Color::WHITE.to_srgba() * brightness).with_alpha(1.0),
        );
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
