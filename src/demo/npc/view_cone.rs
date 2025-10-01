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
            // Close up near perfect vision, high alert
            ViewCone {
                collider: Collider::view_cone(170.0_f32.to_radians(), 170.0_f32.to_radians(), 1.5),
                flags: ViewConeFlags::Active
                    | ViewConeFlags::LowLight
                    | ViewConeFlags::NoAlert0
                    | ViewConeFlags::NoAlert1,
                acuity: 1500.0,
            },
            // Close up near perfect vision, low alert
            ViewCone {
                collider: Collider::view_cone(50.0_f32.to_radians(), 100.0_f32.to_radians(), 1.5),
                flags: ViewConeFlags::Active
                    | ViewConeFlags::LowLight
                    | ViewConeFlags::NoAlert2
                    | ViewConeFlags::NoAlert3,
                acuity: 1500.0,
            },
            ViewCone {
                collider: Collider::view_cone(170.0_f32.to_radians(), 17.0_f32.to_radians(), 1.5),
                flags: ViewConeFlags::Active | ViewConeFlags::NoAlert2 | ViewConeFlags::NoAlert3,
                acuity: 1500.0,
            },
            // Near cone perfect vision
            ViewCone {
                collider: Collider::view_cone(60.0_f32.to_radians(), 90.0_f32.to_radians(), 3.0),
                flags: ViewConeFlags::Active,
                acuity: 200.0,
            },
            // Round the back magic vision
            ViewCone {
                collider: Collider::view_cone(320.0_f32.to_radians(), 90.0_f32.to_radians(), -1.8),
                flags: ViewConeFlags::Active
                    | ViewConeFlags::NoAlert0
                    | ViewConeFlags::NoAlert1
                    | ViewConeFlags::Omni,
                acuity: 70.0,
            },
            // Normal near binocular vision
            ViewCone {
                collider: Collider::view_cone(120.0_f32.to_radians(), 90.0_f32.to_radians(), 6.7),
                flags: ViewConeFlags::Active,
                acuity: 120.0,
            },
            // Mid-range sight
            ViewCone {
                collider: Collider::view_cone(150.0_f32.to_radians(), 70.0_f32.to_radians(), 10.6),
                flags: ViewConeFlags::Active,
                acuity: 80.0,
            },
            /*
            // Long range and wide range peripheral vision
            ViewCone {
                collider: Collider::view_cone(230.0_f32.to_radians(), 70.0_f32.to_radians(), 10.6),
                flags: ViewConeFlags::Active | ViewConeFlags::Periph,
                acuity: 18.2,
            },
            // Long range and high Z
            ViewCone {
                collider: Collider::view_cone(230.0_f32.to_radians(), 110.0_f32.to_radians(), 24.3),
                flags: ViewConeFlags::Active | ViewConeFlags::Periph,
                acuity: 12.1,
            }, */
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

#[derive(Debug)]
pub(crate) struct ViewCone {
    pub(crate) collider: Collider,
    pub(crate) acuity: f32,
    pub(crate) flags: ViewConeFlags,
}

bitflags::bitflags! {
    #[derive(Debug)]
    pub(crate) struct ViewConeFlags: u16 {
        const Active   =  0x01;
        const NoAlert0 =  0x02;
        const NoAlert1 =  0x04;
        const NoAlert2 =  0x08;
        const NoAlert3 =  0x10;

        const AlertnessRestricted = Self::NoAlert0.bits() | Self::NoAlert1.bits() | Self::NoAlert2.bits() | Self::NoAlert3.bits();

        const Periph   =  0x20;
        const Omni     =  0x40;
        const LowLight =  0x80;

        const Behind   = 0x100;
    }
}

const VISIBILITY_ACUITIES: VisibilityAcuities = VisibilityAcuities {
    normal: VisibilityAcuity {
        lighting: 1.0,
        movement: 1.0,
        exposure: 1.0,
    },
    periphery: VisibilityAcuity {
        lighting: 0.3,
        movement: 3.0,
        exposure: 1.0,
    },
    omnidirectional: VisibilityAcuity {
        lighting: 0.8,
        movement: 1.4,
        exposure: 1.2,
    },
    light: VisibilityAcuity {
        lighting: 1.0,
        movement: 0.0,
        exposure: 0.0,
    },
    movement: VisibilityAcuity {
        lighting: 0.0,
        movement: 5.0,
        exposure: 0.0,
    },
    low_light: VisibilityAcuity {
        lighting: 6.0,
        movement: 1.0,
        exposure: 1.0,
    },
};

struct VisibilityAcuities {
    normal: VisibilityAcuity,
    periphery: VisibilityAcuity,
    omnidirectional: VisibilityAcuity,
    light: VisibilityAcuity,
    movement: VisibilityAcuity,
    low_light: VisibilityAcuity,
}

struct VisibilityAcuity {
    lighting: f32,
    movement: f32,
    exposure: f32,
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
            Transform::IDENTITY,
            Visibility::default(),
            NotShadowCaster,
            NotShadowReceiver,
        ));
    }

    Ok(())
}
