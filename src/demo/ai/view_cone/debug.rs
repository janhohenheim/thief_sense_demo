use bevy::{
    asset::RenderAssetUsages,
    color::palettes::tailwind,
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use crate::{
    demo::ai::view_cone::{ViewCone, ViewCones},
    link_head::Head,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DebugViewCones>();
}

/// Guaranteed to have the same order and number of items as [`ViewCones`].
#[derive(Resource, Debug)]
pub(crate) struct DebugViewCones {
    meshes: Vec<Handle<Mesh>>,
    material: Handle<StandardMaterial>,
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
                base_color: Color::from(tailwind::GREEN_400.with_alpha(0.1)),
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
impl ViewCone {
    fn as_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        let (verts, indices) = self
            .collider
            .shape()
            .as_compound()
            .unwrap()
            .shapes()
            .iter()
            .map(|(_, shape)| shape.as_convex_polyhedron().unwrap())
            .fold(
                (Vec::new(), Vec::new()),
                |(mut verts, mut indices), shape| {
                    let (shape_verts, shape_indices) = shape.to_trimesh();
                    let next_index = verts.len() as u32;
                    verts.extend(shape_verts.into_iter().map(Vec3::from));
                    indices.extend(shape_indices.into_iter().flatten().map(|i| i + next_index));
                    (verts, indices)
                },
            );

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
            Transform::IDENTITY,
            Visibility::default(),
            NotShadowCaster,
            NotShadowReceiver,
        ));
    }

    Ok(())
}
