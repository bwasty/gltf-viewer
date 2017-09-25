use std::rc::Rc;
use std::collections::HashMap;
use std::path::Path;

use gltf;
use gltf_importer;

use shader::*;
use render::{Mesh, Node, Texture, Material};

#[derive(Default)]
pub struct Root {
    pub nodes: Vec<Node>,
    pub meshes: Vec<Rc<Mesh>>, // TODO!: use gltf indices; drop Rc?
    pub textures: Vec<Rc<Texture>>,
    pub materials: Vec<Rc<Material>>,
    pub shaders: HashMap<ShaderFlags, Rc<PbrShader>>,
}

impl Root {
    pub fn from_gltf(gltf: &gltf::Gltf, buffers: &gltf_importer::Buffers, base_path: &Path) -> Self {
        let mut root = Root::default();
        let nodes = gltf.nodes()
            .map(|g_node| Node::from_gltf(g_node, &mut root, buffers, base_path))
            .collect();
        root.nodes = nodes;
        root
    }
}
