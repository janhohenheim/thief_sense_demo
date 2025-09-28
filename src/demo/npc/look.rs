use std::f32::consts::TAU;

use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use crate::{
    demo::{
        npc::{Npc, sense::SenseTimer, view_cone::ViewCones},
        player::Player,
        visibility::{AiVisibility, get_or_update_visibility},
    },
    rand_timer::RandTimer,
    third_party::avian::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        look.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
    );
}

fn look(world: &mut World) -> Result {
    let npcs = world
        .query_filtered::<Entity, With<Npc>>()
        .iter(world)
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    for npc in npcs {
        let entities_in_view: Vec<Entity> =
            match world.run_system_cached_with(check_view_cones, npc) {
                Ok(entities) => entities,
                Err(err) => {
                    errors.push(err.to_string());
                    continue;
                }
            };
        for entity in entities_in_view {
            // Process entities in view
            let visibility: AiVisibility =
                match world.run_system_cached_with(get_or_update_visibility, entity) {
                    Ok(visibility) => visibility,
                    Err(err) => {
                        errors.push(err.to_string());
                        continue;
                    }
                };
            info!("Entity {:?}: {:?}", entity, visibility);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(BevyError::from(
            errors.iter().fold(String::new(), |acc, err| acc + &err),
        ))
    }
}

// TODO:
// - use a better name lol
// - return cone metadata: how well is every entity visible?
// - do a raycast before adding the entity to the list, *but* add it to the filter even if the raycast doesn't hit (no need to raycast an occluded entity twice)
fn check_view_cones(
    In(entity): In<Entity>,
    mut npcs: Query<(&Transform, &mut SenseTimer)>,
    player: Single<&Transform, With<Player>>,
    spatial: SpatialQuery,
    view_cones: Res<ViewCones>,
) -> Result<Vec<Entity>> {
    let player_transform = player.into_inner();
    let (npc_transform, mut sense_timer) = npcs.get_mut(entity)?;
    if !sense_timer.is_finished() {
        return Ok(Vec::new());
    }
    const DIST_CUTOFF: f32 = 12.0;
    let ms = if player_transform
        .translation
        .distance_squared(npc_transform.translation)
        > DIST_CUTOFF * DIST_CUTOFF
    {
        500
    } else {
        200
    };
    sense_timer.set_base_time_millis(ms);

    let mut filter = SpatialQueryFilter::default()
        .with_mask(CollisionLayer::AiVisible)
        .with_excluded_entities([entity]);

    let mut entities = Vec::new();
    for view_cone in view_cones.iter() {
        let intersections = spatial.shape_intersections(
            &view_cone.collider,
            npc_transform.translation,
            npc_transform.rotation * Quat::from_rotation_x(TAU / 4.0),
            &filter,
        );
        filter.excluded_entities.extend(&intersections);
        entities.extend(intersections);
    }
    Ok(entities)
}
