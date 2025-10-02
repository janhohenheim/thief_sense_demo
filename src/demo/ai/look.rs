use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use crate::{
    collision_layer::CollisionLayer,
    demo::{
        ai::{
            sense::SenseTimer,
            view_cone::ViewCones,
            visibility::{AiVisibility, get_or_update_visibility},
        },
        npc::Npc,
        player::Player,
    },
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
            errors.iter().fold(String::new(), |acc, err| acc + err),
        ))
    }
}

// TODO:
// - use a better name lol
// - return cone metadata: how well is every entity visible?
fn check_view_cones(
    In(entity): In<Entity>,
    mut npcs: Query<(&Transform, &mut SenseTimer)>,
    player: Single<&Transform, With<Player>>,
    spatial: SpatialQuery,
    view_cones: Res<ViewCones>,
    transforms: Query<&GlobalTransform>,
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
    let mut occlusion_filter = SpatialQueryFilter::default()
        .with_mask(CollisionLayer::Opaque)
        .with_excluded_entities([entity]);
    for view_cone in view_cones.iter() {
        let intersections = spatial.shape_intersections(
            &view_cone.collider,
            npc_transform.translation,
            npc_transform.rotation,
            &filter,
        );
        filter.excluded_entities.extend(&intersections);
        occlusion_filter.excluded_entities.extend(&intersections);
        for intersection in intersections {
            let Ok(transform) = transforms.get(intersection) else {
                continue;
            };
            let translation = transform.translation();
            let (dir, len) = match Dir3::new_and_length(translation - npc_transform.translation) {
                Ok(ok) => ok,
                Err(_) => {
                    warn!("NPC is intersecting with another entity");
                    entities.push(intersection);
                    continue;
                }
            };

            if spatial
                .cast_ray(npc_transform.translation, dir, len, true, &occlusion_filter)
                .is_none()
            {
                entities.push(intersection);
            }
        }
    }
    Ok(entities)
}
