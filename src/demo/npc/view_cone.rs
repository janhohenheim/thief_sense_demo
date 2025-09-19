use std::iter;

use crate::{demo::npc::Npc, third_party::avian::EllipticCone as _};
use anyhow::anyhow;
use avian3d::prelude::*;
use bevy::{prelude::*, scene::SceneInstanceReady};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ViewCones>();
}

#[derive(Resource, Debug, Deref, DerefMut)]
pub(crate) struct ViewCones(pub(crate) Vec<ViewCone>);

impl FromWorld for ViewCones {
    fn from_world(_world: &mut World) -> Self {
        Self(vec![ViewCone {
            collider: Collider::elliptic_cone(0.4, 0.8, 3.5),
        }])
    }
}

#[derive(Debug)]
pub(crate) struct ViewCone {
    pub(crate) collider: Collider,
}

#[derive(Component, Debug)]
#[relationship(relationship_target=Head)]
pub(crate) struct HeadOf(Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship=HeadOf)]
pub(crate) struct Head(Entity);

pub(crate) fn link_head_bone(
    trigger: On<SceneInstanceReady>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    npcs: Query<(), With<Npc>>,
    names: Query<&Name>,
    mut commands: Commands,
) -> Result {
    let Some(npc) = iter::once(trigger.entity)
        .chain(parents.iter_ancestors(trigger.entity))
        .find(|e| npcs.contains(*e))
    else {
        return Err(
            anyhow!("Failed to create view cone attachment: not a descendant of an NPC").into(),
        );
    };
    for child in children.iter_descendants(trigger.entity) {
        let Ok(name) = names.get(child) else {
            continue;
        };
        if name.as_str() != "DEF-head" {
            continue;
        }

        commands.entity(child).insert(HeadOf(npc));

        break;
    }
    Ok(())
}
