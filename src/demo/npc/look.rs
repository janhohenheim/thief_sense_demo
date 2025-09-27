use std::f32::consts::TAU;

use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use crate::{
    demo::{
        npc::{sense::SenseTimer, view_cone::ViewCones},
        player::Player,
    },
    rand_timer::RandTimer,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        look.in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
    );
}

fn look(
    mut npcs: Query<(Entity, &Transform, &mut SenseTimer)>,
    player: Single<&Transform, With<Player>>,
    spatial: SpatialQuery,
    view_cones: Res<ViewCones>,
    names: Query<NameOrEntity>,
) {
    let player_transform = player.into_inner();
    for (npc, npc_transform, mut sense_timer) in &mut npcs {
        if !sense_timer.is_finished() {
            continue;
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
        **sense_timer = RandTimer::from_millis(ms);
        let mut filter = SpatialQueryFilter::default().with_excluded_entities([npc]);

        for view_cone in view_cones.iter() {
            let intersections = spatial.shape_intersections(
                &view_cone.collider,
                npc_transform.translation,
                npc_transform.rotation * Quat::from_rotation_x(TAU / 4.0),
                &filter,
            );
            filter.excluded_entities.extend(&intersections);
            for entity in intersections {
                let name = names.get(entity).unwrap();
                info!("Look: {name}");
            }
        }
    }
}
