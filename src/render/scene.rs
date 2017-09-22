use std::rc::Rc;
use std::path::Path;
use std::collections::HashMap;

use gltf;
use gltf_importer;

use camera::Camera;
use render::{Mesh, Node, Texture, Material};
use render::math::*;
use shader::*;

#[derive(Default)]
pub struct Scene {
    pub name: Option<String>,
    pub nodes: Vec<Node>,
    pub meshes: Vec<Rc<Mesh>>,
    pub textures: Vec<Rc<Texture>>,
    pub materials: Vec<Rc<Material>>,
    pub shaders: HashMap<ShaderFlags, Rc<PbrShader>>,

    pub bounds: Bounds,
}

impl Scene {
    pub fn from_gltf(g_scene: gltf::Scene, buffers: &gltf_importer::Buffers, base_path: &Path) -> Scene {
        let mut scene = Scene {
            name: g_scene.name().map(|s| s.to_owned()),
            ..Default::default()
        };
        scene.nodes = g_scene.nodes()
            .map(|g_node| Node::from_gltf(g_node, &mut scene, buffers, base_path))
            .collect();

        // propagate transforms
        let root_transform = Matrix4::identity();
        for node in &mut scene.nodes {
            node.update_transform(&root_transform);
            node.update_bounds();
            // TODO!: visualize final bounds
            scene.bounds = scene.bounds.union(&node.bounds);
        }

        scene
    }

    // TODO: flatten draw call hierarchy (global Vec<SPrimitive>?)
    pub fn draw(&mut self, camera: &Camera) {
        for node in &mut self.nodes {
            node.draw(camera);
        }
    }
}
