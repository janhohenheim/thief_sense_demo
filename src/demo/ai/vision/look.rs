use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use crate::{
    collision_layer::CollisionLayer,
    demo::{
        ai::{
            awareness::{Alertness, AwarenessLevel},
            sense::SenseTimer,
            vision::{
                view_cone::{ViewCone, ViewCones, VisibilityAcuities},
                visibility::{AiVisibility, get_or_update_visibility},
            },
        },
        npc::Npc,
        player::Player,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        update_all_senses.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
    );
}

fn update_all_senses(world: &mut World) -> Result {
    let npcs = world
        .query_filtered::<Entity, With<Npc>>()
        .iter(world)
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    for npc in npcs {
        if let Err(err) = update_senses(In(npc), world) {
            errors.push(err);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(BevyError::from(
            errors
                .iter()
                .fold(String::new(), |acc, err| acc + &err.to_string()),
        ))
    }
}

fn update_senses(In(npc): In<Entity>, world: &mut World) -> Result {
    let _vision_pulses: Vec<(Entity, AwarenessLevel)> = look(In(npc), world)?;
    Ok(())
}

fn look(In(npc): In<Entity>, world: &mut World) -> Result<Vec<(Entity, AwarenessLevel)>> {
    // TODO: check / update awareness flags (kAIAF_CanRaycast, kAIAF_HaveLOS, etc)
    let entities_in_view: Vec<(Entity, ViewCone)> =
        world.run_system_cached_with(check_view_cones, npc)?;
    let mut pulses = Vec::new();
    let mut errors = Vec::new();
    for (entity, view_cone) in entities_in_view {
        // Process entities in view
        let ai_visibility: AiVisibility =
            match world.run_system_cached_with(get_or_update_visibility, entity) {
                Ok(visibility) => visibility,
                Err(err) => {
                    errors.push(err.to_string());
                    continue;
                }
            };
        let visibility: u8 = match world
            .run_system_cached_with(visibility_to_viewer, (entity, view_cone, ai_visibility))
        {
            Ok(visibility) => visibility,
            Err(err) => {
                errors.push(err.to_string());
                continue;
            }
        };
        let pulse = match visibility {
            v if v < 25 => AwarenessLevel::Lowest,
            v if v < 50 => AwarenessLevel::Low,
            v if v < 75 => AwarenessLevel::Moderate,
            _ => AwarenessLevel::High,
        };
        info!(
            "Entity {:?} ({visibility} -> {pulse:?}): {ai_visibility:?} ",
            entity
        );
        pulses.push((entity, pulse));
    }
    // Todo: don't fail the entire fn when a single pulse fails
    match errors {
        errors if errors.is_empty() => Ok(pulses),
        errors => Err(BevyError::from(
            errors.iter().fold(String::new(), |acc, err| acc + err),
        )),
    }
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
    mut npcs: Query<(&Transform, &mut SenseTimer, &Alertness)>,
    player: Single<&Transform, With<Player>>,
    spatial: SpatialQuery,
    view_cones: Res<ViewCones>,
    transforms: Query<&GlobalTransform>,
) -> Result<Vec<(Entity, ViewCone)>> {
    let player_transform = player.into_inner();
    let (npc_transform, mut sense_timer, alertness) = npcs.get_mut(entity)?;
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
        if !view_cone.flags.active() {
            continue;
        }
        if !view_cone.flags.allowed_by(alertness.0) {
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
                    warn!("NPC is intersecting with another entity");
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
