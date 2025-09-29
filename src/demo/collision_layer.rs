use avian3d::prelude::*;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(mark_static_colliders)
        .add_observer(add_point_light_collider);
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

fn add_point_light_collider(
    add: On<Add, PointLight>,
    point_lights: Query<&PointLight>,
    mut commands: Commands,
) -> Result {
    let point_light = point_lights.get(add.entity)?;
    commands.entity(add.entity).insert((
        // Has no rigid body
        Collider::sphere(point_light.range),
        // Does not collide with anything
        CollisionLayers::new([CollisionLayer::LightSource], LayerMask::NONE),
    ));

    Ok(())
}
