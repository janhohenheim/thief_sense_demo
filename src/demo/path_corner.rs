use bevy::{prelude::*, scene::SceneInstanceReady};
use bevy_trenchbroom::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PathCorner>();
    app.register_type::<NextPathCorner>();
    app.register_type::<PreviousPathCorners>();
}

pub(crate) fn link_path_corners(
    _trigger: Trigger<SceneInstanceReady>,
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
