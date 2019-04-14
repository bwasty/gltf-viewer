use std::rc::Rc;
use std::path::Path;

use gltf;
use yage::gl::GL;

use collision::{Aabb, Union};

use crate::controls::CameraParams;
use crate::render::math::*;
use crate::render::mesh::Mesh;
use crate::render::Root;
use crate::render::camera::Camera;
use crate::importdata::ImportData;

pub struct Node {
    pub index: usize, // glTF index
    pub children: Vec<usize>,
    pub mesh: Option<Rc<Mesh>>,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub translation: Vector3,
    // TODO: weights
    // weights_id: usize,
    pub camera: Option<Camera>,
    pub name: Option<String>,

    pub final_transform: Matrix4, // including parent transforms
    pub bounds: Aabb3,
}


impl Node {
    pub fn from_gltf(
        gl: &Rc<GL>,
        g_node: &gltf::Node<'_>,
        root: &mut Root,
        imp: &ImportData,
        base_path: &Path
    ) -> Node {
        let (trans, rot, scale) = g_node.transform().decomposed();
        let r = rot;
        let rotation = Quaternion::new(r[3], r[0], r[1], r[2]); // NOTE: different element order!

        let mut mesh = None;
        if let Some(g_mesh) = g_node.mesh() {
            if let Some(existing_mesh) = root.meshes.iter().find(|mesh| (***mesh).index == g_mesh.index()) {
                mesh = Some(Rc::clone(existing_mesh));
            }

            if mesh.is_none() { // not using else due to borrow-checking madness
                mesh = Some(Rc::new(Mesh::from_gltf(gl, &g_mesh, root, imp, base_path)));
                root.meshes.push(mesh.clone().unwrap());
            }
        }
        let children: Vec<_> = g_node.children()
                .map(|g_node| g_node.index())
                .collect();

        Node {
            index: g_node.index(),
            children,
            mesh,
            rotation,
            scale: scale.into(),
            translation: trans.into(),
            camera: g_node.camera().as_ref().map(Camera::from_gltf),
            name: g_node.name().map(|s| s.into()),

            final_transform: Matrix4::identity(),

            bounds: Aabb3::zero(),
        }
    }

    pub fn update_transform(&mut self, root: &mut Root, parent_transform: &Matrix4) {
        self.final_transform = *parent_transform;

        // TODO: cache local tranform when adding animations?
        self.final_transform = self.final_transform *
            Matrix4::from_translation(self.translation) *
            Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z) *
            Matrix4::from(self.rotation);

        for node_id in &self.children {
            let node = root.unsafe_get_node_mut(*node_id);
            node.update_transform(root, &self.final_transform);
        }
    }

    /// Should be called after update_transforms
    pub fn update_bounds(&mut self, root: &mut Root) {
        self.bounds = Aabb3::zero();
        if let Some(ref mesh) = self.mesh {
            self.bounds = mesh.bounds
                .transform(&self.final_transform);
        }

        for node_id in &self.children {
            let node = root.unsafe_get_node_mut(*node_id);
            node.update_bounds(root);
            self.bounds = self.bounds.union(&node.bounds);
        }
    }

    pub fn draw(&mut self, root: &mut Root, cam_params: &CameraParams) {
        if let Some(ref mesh) = self.mesh {
            let mvp_matrix = cam_params.projection_matrix * cam_params.view_matrix * self.final_transform;

            (*mesh).draw(&self.final_transform, &mvp_matrix, &cam_params.position);
        }
        for node_id in &self.children {
            let node = root.unsafe_get_node_mut(*node_id);
            node.draw(root, cam_params);
        }
    }
}
