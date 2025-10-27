use bevy::{picking::pointer::PointerInteraction, prelude::*};
use bevy_landmass::{
    AgentSettings, AgentTarget3d, Archipelago3d, FromAgentRadius as _, PointSampleDistance3d,
};
use bevy_ui_anchor::{AnchorPoint, AnchorUiConfig, AnchoredUiNodes};

use crate::{
    cpu_lighting::estimate_tone_mapped_lighting,
    demo::player::{PLAYER_RADIUS, PLAYER_RUN_SPEED, PLAYER_WALK_SPEED, Player},
    third_party::landmass::Agent,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_pointer_gizmo_lighting, draw_pointer_gizmo).chain(),
    );
    app.add_observer(move_player);
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((
            PointerGizmoLighting(1.0),
            AnchoredUiNodes::spawn_one((
                AnchorUiConfig {
                    anchorpoint: AnchorPoint::bottommid(),
                    offset: Some(Vec3::new(0.0, 0.5, 0.0)),
                    ..default()
                },
                children![(Text::default(), GizmoText, Pickable::IGNORE,)],
                Pickable::IGNORE,
            )),
        ));
    });
}

#[derive(Component)]
struct GizmoText;

#[derive(Component)]
#[require(Transform)]
struct PointerGizmoLighting(f32);

/// A system that draws hit indicators for every pointer.
fn update_pointer_gizmo_lighting(world: &mut World) -> Result {
    let Some(point) = world
        .query::<&PointerInteraction>()
        .query(world)
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position)
        .next()
    else {
        return Ok(());
    };

    let point = point + Vec3::Y * 0.05;
    {
        let mut transform = world
            .query_filtered::<&mut Transform, With<PointerGizmoLighting>>()
            .single_mut(world)?;
        transform.translation = point;
    }
    let entity = world
        .query_filtered::<Entity, With<PointerGizmoLighting>>()
        .single(world)?;
    let lighting = world.run_system_cached_with(estimate_tone_mapped_lighting, entity)?;
    world
        .query::<&mut PointerGizmoLighting>()
        .single_mut(world)?
        .0 = lighting;

    Ok(())
}

fn draw_pointer_gizmo(
    pointer: Single<(&GlobalTransform, &PointerGizmoLighting)>,
    mut text: Single<&mut Text, With<GizmoText>>,
    mut gizmos: Gizmos,
) {
    let (transform, lighting) = pointer.into_inner();
    gizmos.sphere(
        transform.translation(),
        0.5,
        (Color::WHITE.to_srgba() * lighting.0).with_alpha(1.0),
    );
    ***text = format!("{:.3}", lighting.0);
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
