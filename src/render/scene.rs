use gltf;

use render::Node;
use shader::Shader;

pub struct Scene {
    pub name: Option<String>,
    pub nodes: Vec<Node>,
}

impl Scene {
    pub fn from_gltf(g_scene: gltf::scene::Scene) -> Scene {
        Scene {
            name: g_scene.name().map(|s| s.into()),
            nodes: g_scene.nodes().map(Node::from_gltf).collect()
        }
    }

    // TODO!: flatten draw call hierarchy (global Vec<Primitive>?)
    pub fn draw(&self, shader: &Shader) {
        for node in &self.nodes {
            node.draw(shader);
        }
    }
}