use bevy::{ecs::relationship::Relationship, prelude::*, scene::SceneInstanceReady};
use bevy_trenchbroom::prelude::*;

use crate::demo::{
    target::{Target, TargetOf},
    target_after::TargetAfter,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PathCorner>();
    app.register_type::<NextPathCorner>();
    app.register_type::<PreviousPathCorners>();
    app.add_systems(FixedUpdate, update_targets_to_path_corners_on_arrival);
}

pub(crate) fn link_path_corners(
    _trigger: On<SceneInstanceReady>,
    corners: Query<(Entity, &PathCorner)>,
    mut commands: Commands,
) {
    for (entity, corner) in corners.iter() {
        let Some(next_target) = corner.target.as_ref() else {
            continue;
        };
        let Some(next_target_entity) = corners
            .iter()
            .find(|(_, c)| &c.targetname == next_target)
            .map(|(e, _)| e)
        else {
            error!(
                "Failed to link path corner {} to {}: Did not find target",
                corner.targetname, next_target
            );
            continue;
        };
        commands
            .entity(entity)
            .insert(NextPathCorner(next_target_entity));
    }
}

fn update_targets_to_path_corners_on_arrival(
    corners: Query<(&Transform, &TargetOf, &PathCorner, Option<&NextPathCorner>)>,
    transforms: Query<&Transform>,
    mut commands: Commands,
) {
    for (corner_transform, target_of, current_corner, next_corner) in corners.iter() {
        for ai_entity in target_of.iter() {
            let Ok(ai_transform) = transforms.get(ai_entity) else {
                error!("Failed to get AI transform",);
                continue;
            };
            let distance = corner_transform
                .translation
                .distance(ai_transform.translation);
            if distance < 1.0 {
                commands.entity(ai_entity).remove::<Target>();
                if let Some(next_corner) = next_corner {
                    let target = Target(next_corner.get());
                    if current_corner.wait == 0.0 {
                        commands.entity(ai_entity).insert(target);
                    } else {
                        commands
                            .entity(ai_entity)
                            .insert(TargetAfter::new(target, current_corner.wait));
                    }
                }
            }
        }
    }
}

#[derive(Default)]
#[point_class(color(0 0 255))]
pub(crate) struct PathCorner {
    #[class(must_set)]
    #[class(title = "Name")]
    pub(crate) targetname: String,
    #[class(title = "Next target")]
    pub(crate) target: Option<String>,
    pub(crate) wait: f32,
}

#[derive(Component, Reflect, Clone, Deref)]
#[reflect(Component)]
#[relationship_target(relationship = NextPathCorner)]
pub(crate) struct PreviousPathCorners(Vec<Entity>);

#[derive(Component, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = PreviousPathCorners)]
pub(crate) struct NextPathCorner(pub(crate) Entity);
