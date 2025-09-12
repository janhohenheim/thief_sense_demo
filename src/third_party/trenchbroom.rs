use bevy::prelude::*;
use bevy_trenchbroom::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TrenchBroomPlugins(
        TrenchBroomConfig::new("Thief Sense Demo").default_solid_spawn_hooks(|| {
            SpawnHooks::new()
                .convex_collider()
                .smooth_by_default_angle()
        }),
    ));
}
