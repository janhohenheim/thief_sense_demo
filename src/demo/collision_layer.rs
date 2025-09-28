use avian3d::prelude::*;
use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use bevy_trenchbroom::geometry::Brushes;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(mark_static_colliders);
}

#[derive(Debug, PhysicsLayer, Default)]
pub(crate) enum CollisionLayer {
    #[default]
    Default,
    AiVisible,
    LightSource,
    Transparent,
    Static,
}

fn mark_static_colliders(
    add: On<Add, RigidBodyColliders>,
    rigid_body: Query<(&RigidBody, &RigidBodyColliders)>,
    mut layers: Query<&mut CollisionLayers>,
    mut commands: Commands,
) -> Result {
    let (rigid_body, colliders) = rigid_body.get(add.entity)?;
    if !rigid_body.is_static() {
        return Ok(());
    }

    for entity in colliders.iter() {
        if let Ok(mut layer) = layers.get_mut(entity) {
            layer.memberships.add(CollisionLayer::Static);
        } else {
            commands.entity(entity).insert(CollisionLayers::new(
                [CollisionLayer::Default, CollisionLayer::Static],
                [CollisionLayer::Default],
            ));
        }
    }
    Ok(())
}
