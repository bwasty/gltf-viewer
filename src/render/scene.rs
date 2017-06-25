use render::Node;

pub struct Scene {
    pub gltf_index: usize,
    pub name: Option<String>,
    pub nodes: Vec<Node>,
}
