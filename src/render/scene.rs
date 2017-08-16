use std::rc::Rc;
// use std::time::Instant;

use gltf;

use render::{Mesh, Node, Texture, Material};
use render::math::*;
use shader::Shader;
// use utils::print_elapsed;

#[derive(Default)]
pub struct Scene {
    pub name: Option<String>,
    pub nodes: Vec<Node>,
    pub meshes: Vec<Rc<Mesh>>,
    pub textures: Vec<Rc<Texture>>,
    pub materials: Vec<Rc<Material>>,

    pub bounds: Bounds,
}

impl Scene {
    pub fn from_gltf(g_scene: gltf::Loaded<gltf::Scene>) -> Scene {
        let mut scene = Scene {
            name: g_scene.name().map(|s| s.to_owned()),
            ..Default::default()
        };
        scene.nodes = g_scene.nodes()
            .map(|g_node| Node::from_gltf(g_node, &mut scene))
            .collect();

        // propagate transforms
        // let start_time = Instant::now();
        let root_transform = Matrix4::identity();
        for node in &mut scene.nodes {
            node.update_transform(&root_transform);
            node.update_bounds();
            // TODO!: visualize final bounds
            scene.bounds = scene.bounds.union(&node.bounds);
        }
        // print_elapsed("propagate transforms", &start_time);
        println!("Scene: {:?}", scene.bounds);

        scene
    }

    // TODO: flatten draw call hierarchy (global Vec<SPrimitive>?)
    pub fn draw(&mut self, shader: &mut Shader) {
        for node in &mut self.nodes {
            node.draw(shader);
        }
    }
}
