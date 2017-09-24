use std::rc::Rc;
use std::path::Path;

use gltf;
use gltf_importer;

use camera::CameraControls;
use render::math::*;
use render::mesh::Mesh;
use render::scene::Scene;
use render::camera::Camera;

pub struct Node {
    // TODO!!: handle camera in Node
    pub children: Vec<Node>,
    pub matrix: Matrix4,
    pub mesh: Option<Rc<Mesh>>,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub translation: Vector3,
    // TODO: weights
    // weights_id: usize,
    pub camera: Option<Camera>,
    pub name: Option<String>,

    final_transform: Matrix4, // including parent transforms
    model_loc: Option<i32>,

    pub bounds: Bounds,
}


impl Node {
    // TODO!: refactor transformations using mint and non-deprecated functions
    #[allow(deprecated)]
    pub fn from_gltf(
        g_node: gltf::Node,
        scene: &mut Scene,
        buffers: &gltf_importer::Buffers,
        base_path: &Path
    ) -> Node {
        // convert matrix in 3 steps due to type system weirdness
        let matrix = &g_node.matrix();
        let matrix: &Matrix4 = matrix.into();
        let matrix = *matrix;

        let r = &g_node.rotation();
        let rotation = Quaternion::new(r[3], r[0], r[1], r[2]); // NOTE: different element order!

        let mut mesh = None;
        if let Some(g_mesh) = g_node.mesh() {
            if let Some(existing_mesh) = scene.meshes.iter().find(|mesh| (***mesh).index == g_mesh.index()) {
                mesh = Some(Rc::clone(existing_mesh));
            }

            if mesh.is_none() { // not using else due to borrow-checking madness
                mesh = Some(Rc::new(Mesh::from_gltf(g_mesh, scene, buffers, base_path)));
                scene.meshes.push(mesh.clone().unwrap());
            }
        }
        let children: Vec<_> = g_node.children()
                .map(|g_node| Node::from_gltf(g_node, scene, buffers, base_path))
                .collect();

        Node {
            children,
            matrix,
            mesh,
            rotation,
            scale: g_node.scale().into(),
            translation: g_node.translation().into(),
            camera: g_node.camera().as_ref().map(Camera::from_gltf),
            name: g_node.name().map(|s| s.into()),

            final_transform: Matrix4::identity(),
            model_loc: None,

            bounds: Bounds::default(),
        }
    }

    pub fn update_transform(&mut self, parent_transform: &Matrix4) {
        self.final_transform = *parent_transform;

        if !self.matrix.is_identity() {
            self.final_transform = self.final_transform * self.matrix;
        }
        else {
            // TODO?: detect if all default and set None? does NOT happen for any sample model
            self.final_transform = self.final_transform *
                Matrix4::from_translation(self.translation) *
                Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z) *
                Matrix4::from(self.rotation);
        }

        for node in &mut self.children {
            node.update_transform(&self.final_transform);
        }
    }

    /// Should be called after update_transforms
    pub fn update_bounds(&mut self) {
        self.bounds = Default::default();
        if let Some(ref mesh) = self.mesh {
            self.bounds = mesh.bounds
                .transform(&self.final_transform);
        }
        else if self.children.is_empty() {
            // Cameras (others?) have neither mesh nor children. Their position is the origin
            // TODO!: are there other cases? Do bounds matter for cameras?
            self.bounds = Bounds { min: Vector3::zero(), max: Vector3::zero() };
            self.bounds = self.bounds.transform(&self.final_transform);
        }
        else {
            for node in &mut self.children {
                node.update_bounds();
                self.bounds = self.bounds.union(&node.bounds);
            }
        }
    }

    pub fn draw(&mut self, controls: &CameraControls) {
        if let Some(ref mesh) = self.mesh {
            let mvp_matrix = controls.camera.projection_matrix * controls.view_matrix() * self.final_transform;

            (*mesh).draw(&self.final_transform, &mvp_matrix, &controls.position.to_vec());
        }
        for node in &mut self.children {
            node.draw(controls);
        }
    }
}
