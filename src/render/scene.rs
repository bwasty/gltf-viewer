use std::rc::Rc;
use std::time::Instant;

use gltf;

use render::{Mesh, Node, Texture};
use render::math::*;
use shader::Shader;
use utils::print_elapsed;

#[derive(Default)]
pub struct Scene {
    pub name: Option<String>,
    pub nodes: Vec<Node>,
    pub meshes: Vec<Rc<Mesh>>,
    pub textures: Vec<Rc<Texture>>,
    // TODO!: Materials
}

impl Scene {
    pub fn from_gltf(g_scene: gltf::scene::Scene) -> Scene {
        let mut scene = Scene {
            name: g_scene.name().map(|s| s.to_owned()),
            ..Default::default()
        };
        scene.nodes = g_scene.nodes()
            .map(|g_node| Node::from_gltf(g_node, &mut scene))
            .collect();

        // propagate transforms
        let start_time = Instant::now();
        let root_transform = Matrix4::identity();
        for node in &mut scene.nodes {
            node.update_transform(&root_transform);
        }
        print_elapsed("propagate transforms", &start_time);

        scene
    }

    // TODO: flatten draw call hierarchy (global Vec<SPrimitive>?)
    pub fn draw(&mut self, shader: &mut Shader) {
        for node in &mut self.nodes {
            node.draw(shader);
        }
    }
}
