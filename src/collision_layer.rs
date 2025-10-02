use avian3d::prelude::*;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(mark_static_colliders)
        .add_observer(add_point_light_collider)
        .add_observer(add_spot_light_collider);
}

#[derive(Debug, PhysicsLayer, Default)]
pub(crate) enum CollisionLayer {
    #[default]
    Default,
    AiVisible,
    LightSource,
    Opaque,
    Static,
}

/// Add this to an entity as a required component to ensure that it does not get marked as [`CollisionLayer::Opaque`]
#[derive(Component)]
pub(crate) struct Transparent;

fn mark_static_colliders(
    add: On<Add, RigidBodyColliders>,
    rigid_body: Query<(&RigidBody, &RigidBodyColliders, Has<Transparent>)>,
    mut layers: Query<&mut CollisionLayers>,
    mut commands: Commands,
) -> Result {
    let (rigid_body, colliders, is_transparent) = rigid_body.get(add.entity)?;
    let is_static = rigid_body.is_static();
    if !is_static && is_transparent {
        return Ok(());
    }

    for entity in colliders.iter() {
        if let Ok(mut layer) = layers.get_mut(entity) {
            if is_static {
                layer.memberships.add(CollisionLayer::Static);
            }
            if !is_transparent {
                layer.memberships.add(CollisionLayer::Opaque);
            }
        } else {
            let mut layers = CollisionLayers::DEFAULT;
            if is_static {
                layers.memberships.add(CollisionLayer::Static);
            }
            if !is_transparent {
                layers.memberships.add(CollisionLayer::Opaque);
            }
            commands.entity(entity).insert(layers);
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

fn add_spot_light_collider(
    add: On<Add, SpotLight>,
    spot_lights: Query<&SpotLight>,
    mut commands: Commands,
) -> Result {
    let spot_light = spot_lights.get(add.entity)?;
    commands.entity(add.entity).insert((
        // Has no rigid body
        Collider::sphere(spot_light.range),
        // Does not collide with anything
        CollisionLayers::new([CollisionLayer::LightSource], LayerMask::NONE),
    ));

    Ok(())
}
