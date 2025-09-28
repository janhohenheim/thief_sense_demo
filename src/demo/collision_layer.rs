use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_trenchbroom::geometry::Brushes;

pub(super) fn plugin(app: &mut App) {
    app.register_required_components_with::<Brushes, _>(|| {
        CollisionLayers::new(
            [CollisionLayer::Default, CollisionLayer::Brush],
            [CollisionLayer::Default],
        )
    });
}

#[derive(Debug, PhysicsLayer, Default)]
pub(crate) enum CollisionLayer {
    #[default]
    Default,
    AiVisible,
    LightSource,
    Transparent,
    Brush,
}
