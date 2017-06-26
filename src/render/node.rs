use std::rc::Rc;

use gltf;

use render::math::*;
use render::mesh::Mesh;

pub struct Node {
    // TODO: camera?
    pub children: Vec<Node>,
    pub matrix: Matrix4,
    pub mesh: Option<Rc<Mesh>>,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub translation: Vector3,
    // TODO
    // weights_id: usize,
    pub name: Option<String>,
}

impl Node {
    pub fn from_gltf(g_node: gltf::scene::Node) -> Node {
        Node {
            children: g_node.children().map(Node::from_gltf).collect(),
            // TODO!!
            // matrix: Matrix4::from(&g_node.matrix()),
            matrix: Matrix4::identity(),
            mesh: g_node.mesh().map(|g_mesh| Rc::new(Mesh::from_gltf(g_mesh))),
            rotation: Quaternion::from(g_node.rotation()),
            scale: Vector3::from(g_node.scale()),
            translation: Vector3::from(g_node.translation()),
            name: g_node.name().map(|s| s.into()),
        }
    }
}
