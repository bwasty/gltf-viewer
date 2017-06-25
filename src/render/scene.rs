use gltf;

use render::Node;

pub struct Scene {
    pub name: Option<String>,
    pub nodes: Vec<Node>,
}

impl Scene {
    pub fn from_gltf(scene: gltf::scene::Scene) -> Scene {
        Scene {
            name: scene.name().map(|s| s.into()),
            nodes: scene.nodes().map(Node::from_gltf).collect()
        }
    }
}
