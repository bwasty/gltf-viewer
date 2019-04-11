use std::rc::Rc;
use std::collections::HashMap;
use std::path::Path;

use yage::gl::GL;

use crate::shader::*;
use crate::render::{Mesh, Node, Material};
use crate::render::texture::Texture;
use crate::importdata::ImportData;

#[derive(Default)]
pub struct Root<'a> {
    pub nodes: Vec<Node<'a>>,
    pub meshes: Vec<Rc<Mesh<'a>>>, // TODO!: use gltf indices; drop Rc?
    pub textures: Vec<Rc<Texture>>,
    pub materials: Vec<Rc<Material>>,
    pub shaders: HashMap<ShaderFlags, Rc<PbrShader>>,

    pub camera_nodes: Vec<usize>, // indices of camera nodes
    // TODO!: joint_nodes, mesh_nodes?
}

impl<'a> Root<'a> {
    pub fn from_gltf(gl: &'a GL, imp: &ImportData, base_path: &Path) -> Root<'a> {
        let mut root: Root<'a> = Root::default();
        let nodes = imp.doc.nodes()
            .map(|g_node| Node::from_gltf(gl, &g_node, &mut root, imp, base_path))
            .collect();
        root.nodes = nodes;
        root.camera_nodes = root.nodes.iter()
            .filter(|node| node.camera.is_some())
            .map(|node| node.index)
            .collect();
        root
    }

    /// Get a mutable reference to a node without borrowing `Self` or `Self::nodes`.
    /// Safe for tree traversal (visiting each node ONCE and NOT keeping a reference)
    /// as long as the gltf is valid, i.e. the scene actually is a tree.
    pub fn unsafe_get_node_mut(&mut self, index: usize) ->&'a mut Node {
        unsafe {
            &mut *(&mut self.nodes[index] as *mut Node)
        }
    }

    /// Note: index refers to the vec of camera node indices!
    pub fn get_camera_node(&self, index: usize) -> &Node {
        &self.nodes[self.camera_nodes[index]]
    }
}
