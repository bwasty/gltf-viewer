use std::rc::Rc;

use gltf;

use render::math::*;
use render::mesh::Mesh;

pub struct Node {
    // TODO: camera?
    pub children: Vec<Node>,
    pub matrix: Matrix4,
    pub mesh: Rc<Mesh>,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub translation: Vector3,
    // TODO
    // weights_id: usize,
    pub name: Option<String>,
}

impl Node {
    pub fn from_gltf(node: gltf::scene::Node) -> Node {
        unimplemented!()
    }
}
