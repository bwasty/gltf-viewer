// use std::rc::Rc;
use std::path::Path;

use collision::{Aabb, Aabb3, Union};

use gltf;

use crate::render::math::*;
use crate::render::{Primitive, Root};
use crate::importdata::ImportData;

use crate::platform::{PrimitiveHelpers};

pub struct Mesh {
    pub index: usize, // glTF index
    pub primitives: Vec<Primitive>,
    // TODO: weights
    // pub weights: Vec<Rc<?>>
    pub name: Option<String>,

    pub bounds: Aabb3<f32>,
}

impl Mesh {
    pub fn from_gltf(
        g_mesh: &gltf::Mesh<'_>,
        root: &mut Root,
        imp: &ImportData,
        base_path: &Path,
    ) -> Mesh {
        let primitives: Vec<Primitive> = g_mesh.primitives()
            .enumerate()
            .map(|(i, g_prim)| {
                Primitive::from_gltf(&g_prim, i, g_mesh.index(), root, imp, base_path)
            })
            .collect();

        let bounds = primitives.iter()
            .fold(Aabb3::zero(), |bounds, prim| prim.bounds.union(&bounds));

        Mesh {
            index: g_mesh.index(),
            primitives,
            name: g_mesh.name().map(|s| s.into()),
            bounds,
        }
    }

    pub fn draw(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3) {
        for primitive in &self.primitives {
            unsafe { primitive.draw(model_matrix, mvp_matrix, camera_position) }
        }
    }
}
