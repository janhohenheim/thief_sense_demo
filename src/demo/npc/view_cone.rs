use std::f32::consts::TAU;

use crate::{demo::link_head::Head, third_party::avian::EllipticCone as _};
use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    color::palettes::tailwind,
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ViewCones>()
        .init_resource::<DebugViewCones>();
}

#[derive(Resource, Debug, Deref, DerefMut)]
pub(crate) struct ViewCones(pub(crate) Vec<ViewCone>);

/// Guaranteed to have the same order and number of items as [`ViewCones`].
#[derive(Resource, Debug)]
pub(crate) struct DebugViewCones {
    meshes: Vec<Handle<Mesh>>,
    material: Handle<StandardMaterial>,
}

impl FromWorld for ViewCones {
    fn from_world(_world: &mut World) -> Self {
        Self(vec![
            ViewCone {
                collider: Collider::elliptic_cone(0.4, 0.8, 3.5),
            },
            ViewCone {
                collider: Collider::elliptic_cone(1.2, 1.2, 2.5),
            },
        ])
    }
}

impl FromWorld for DebugViewCones {
    fn from_world(world: &mut World) -> Self {
        let mut view_cone_meshes = Vec::new();
        world.resource_scope(|world: &mut World, view_cones: Mut<ViewCones>| {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            for view_cone in view_cones.iter() {
                let handle = meshes.add(view_cone.as_mesh());
                view_cone_meshes.push(handle);
            }
        });
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::from(tailwind::GREEN_400.with_alpha(0.2)),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });
        Self {
            meshes: view_cone_meshes,
            material,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ViewCone {
    pub(crate) collider: Collider,
}
impl ViewCone {
    fn as_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        let (verts, indices) = self
            .collider
            .shape()
            .as_convex_polyhedron()
            .unwrap()
            .to_trimesh();
        let verts = verts.into_iter().map(Vec3::from).collect::<Vec<_>>();
        let indices = indices.into_iter().flatten().collect();

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }
}

pub(crate) fn add_debug_view_cones(
    head: On<Add, Head>,
    heads: Query<&Head>,
    mut commands: Commands,
    debug_view_cones: Res<DebugViewCones>,
) -> Result {
    let head = heads.get(head.entity)?;
    // is this really the best way? :hmm:
    let head = head.iter().next().unwrap();
    for mesh in &debug_view_cones.meshes {
        commands.entity(head).with_child((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(debug_view_cones.material.clone()),
            Transform::from_rotation(Quat::from_rotation_x(TAU / 4.0)),
            Visibility::default(),
            NotShadowCaster,
            NotShadowReceiver,
        ));
    }

    Ok(())
}
