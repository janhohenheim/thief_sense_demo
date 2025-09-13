use bevy::{prelude::*, scene::SceneInstanceReady};
use bevy_trenchbroom::prelude::*;

use crate::demo::path_corner::PathCorner;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<TargetBase>();
    app.register_type::<TargetOf>();
    app.register_type::<Target>();
}

pub(crate) fn link_targets(
    _trigger: Trigger<SceneInstanceReady>,
    target_bases: Query<(Entity, &TargetBase)>,
    corners: Query<(Entity, &PathCorner)>,
    mut commands: Commands,
) {
    for (entity, target_base) in target_bases.iter() {
        let Some(target_name) = target_base.target.as_ref() else {
            continue;
        };
        let Some(target_entity) = corners
            .iter()
            .find(|(_, c)| &c.targetname == target_name)
            .map(|(e, _)| e)
        else {
            error!("Failed to link target {target_name}: Did not find target");
            continue;
        };
        commands.entity(entity).insert(Target(target_entity));
    }
}

#[derive(Default)]
#[base_class]
pub(crate) struct TargetBase {
    target: Option<String>,
}

#[derive(Component, Reflect, Clone, Deref)]
#[reflect(Component)]
#[relationship_target(relationship = Target)]
pub(crate) struct TargetOf(Vec<Entity>);

#[derive(Component, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = TargetOf)]
pub(crate) struct Target(pub(crate) Entity);
