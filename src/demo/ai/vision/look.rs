use avian3d::prelude::{ColliderOf, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use crate::{
    collision_layer::CollisionLayer,
    demo::ai::{
        alertness::Alertness,
        awareness::AwarenessLevel,
        debug::DebugVision,
        vision::{
            view_cone::{ViewCone, ViewCones, VisibilityAcuities},
            visibility::{AiVisibility, get_or_update_visibility},
        },
    },
};

pub(super) fn plugin(_app: &mut App) {}

pub(crate) fn look(
    In(npc): In<Entity>,
    world: &mut World,
) -> Result<Vec<(Entity, AwarenessLevel)>> {
    // The original only checks the view cones for potential targets that already passed a raycast,
    // but I think the performance difference is negligible since we are doing cheap shape_intersections.
    let colliders_in_view: Vec<(Entity, ViewCone)> =
        world.run_system_cached_with(check_view_cones, npc)?;
    let mut pulses = Vec::new();
    for (collider, view_cone) in colliders_in_view {
        // Process entities in view
        let ai_visibility: AiVisibility =
            match world.run_system_cached_with(get_or_update_visibility, collider) {
                Ok(visibility) => visibility,
                Err(err) => {
                    error!("{err}");
                    continue;
                }
            };
        let visibility: u8 = match world
            .run_system_cached_with(visibility_to_viewer, (collider, view_cone, ai_visibility))
        {
            Ok(visibility) => visibility,
            Err(err) => {
                error!("{err}");
                continue;
            }
        };
        let pulse = match visibility {
            v if v < 25 => AwarenessLevel::Lowest,
            v if v < 50 => AwarenessLevel::Low,
            v if v < 75 => AwarenessLevel::Moderate,
            _ => AwarenessLevel::High,
        };
        let entity = match world.entity(collider).get::<ColliderOf>() {
            Some(collider_of) => collider_of.body,
            None => {
                error!("Visible entity does not belong to a rigid body");
                continue;
            }
        };
        world.entity_mut(npc).insert(DebugVision {
            entity,
            visibility: ai_visibility,
        });
        pulses.push((entity, pulse));
    }
    Ok(pulses)
}

fn visibility_to_viewer(
    In((_entity, view_cone, visibility)): In<(Entity, ViewCone, AiVisibility)>,
    acuities: Res<VisibilityAcuities>,
) -> Result<u8> {
    let acuity = acuities.for_cone(view_cone.flags);
    let mut visibility_to_viewer = visibility.lighting as f32 * acuity.lighting
        + visibility.movement as f32 * acuity.movement
        + visibility.exposure as f32 * acuity.exposure;
    visibility_to_viewer = visibility_to_viewer.max(1.0) * view_cone.acuity;

    // TODO: factor in visibility types
    Ok(visibility_to_viewer.clamp(0.0, 100.0) as u8)
}

/// Returns a list of entities that the NPC can see, along with their view cones.
/// Cloning view cones around like this is surprisingly cheap because they use Arcs internally.
fn check_view_cones(
    In(entity): In<Entity>,
    mut npcs: Query<(&Transform, &Alertness)>,
    spatial: SpatialQuery,
    view_cones: Res<ViewCones>,
    transforms: Query<&GlobalTransform>,
) -> Result<Vec<(Entity, ViewCone)>> {
    let (npc_transform, alertness) = npcs.get_mut(entity)?;

    let mut filter = SpatialQueryFilter::default()
        .with_mask(CollisionLayer::AiVisible)
        .with_excluded_entities([entity]);

    let mut entities = Vec::new();
    let mut occlusion_filter = SpatialQueryFilter::default()
        .with_mask(CollisionLayer::Opaque)
        .with_excluded_entities([entity]);
    for view_cone in view_cones.iter() {
        if !view_cone.flags.active() {
            continue;
        }
        if !view_cone.flags.allowed_by(alertness.level) {
            continue;
        }
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
                    warn!("NPC is at the same position as another entity");
                    entities.push((intersection, view_cone.clone()));
                    continue;
                }
            };

            if spatial
                .cast_ray(npc_transform.translation, dir, len, true, &occlusion_filter)
                .is_none()
            {
                entities.push((intersection, view_cone.clone()));
            }
        }
    }
    Ok(entities)
}
