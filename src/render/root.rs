use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::path::Path;

use gltf;
use gltf_importer;

use shader::*;
use render::{Mesh, Node, Texture, Material};

#[derive(Default)]
pub struct Root {
    pub nodes: Vec<RefCell<Node>>,
    pub meshes: Vec<Rc<Mesh>>, // TODO!: use gltf indices; drop Rc?
    pub textures: Vec<Rc<Texture>>,
    pub materials: Vec<Rc<Material>>,
    pub shaders: HashMap<ShaderFlags, Rc<PbrShader>>,

    pub camera_nodes: Vec<usize>, // indices of camera nodes
    // TODO!: joint_nodes, mesh_nodes?
}

impl Root {
    pub fn from_gltf(gltf: &gltf::Gltf, buffers: &gltf_importer::Buffers, base_path: &Path) -> Self {
        let mut root = Root::default();
        let nodes = gltf.nodes()
            .map(|g_node| Node::from_gltf(g_node, &mut root, buffers, base_path).into())
            .collect();
        root.nodes = nodes;
        root.camera_nodes = root.nodes.iter()
            .filter(|node| node.borrow().camera.is_some())
            .map(|node| node.borrow().index)
            .collect();
        root
    }
}
