use std::rc::Rc;

use gltf;

use render::Mesh;
use render::Node;
use render::math::*;
use shader::Shader;

#[derive(Default)]
pub struct Scene {
    pub name: Option<String>,
    pub nodes: Vec<Node>,
    pub meshes: Vec<Rc<Mesh>>,
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
        scene
    }

    // TODO: flatten draw call hierarchy (global Vec<Primitive>?)
    pub fn draw(&mut self, shader: &mut Shader) {
        let model_matrix = Matrix4::identity();
        for node in &mut self.nodes {
            node.draw(shader, &model_matrix);
        }
    }
}
