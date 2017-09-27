
use gltf;

use camera::CameraControls;
use render::{Root};
use render::math::*;

#[derive(Default)]
pub struct Scene {
    pub name: Option<String>,
    pub nodes: Vec<usize>,
    pub bounds: Bounds,
}

impl Scene {
    pub fn from_gltf(g_scene: gltf::Scene, root: &mut Root) -> Scene {
        let mut scene = Scene {
            name: g_scene.name().map(|s| s.to_owned()),
            ..Default::default()
        };
        scene.nodes = g_scene.nodes()
            .map(|g_node| g_node.index())
            .collect();

        // propagate transforms
        let root_transform = Matrix4::identity();
        for node_id in &scene.nodes {
            let mut node = root.nodes[*node_id].borrow_mut();
            node.update_transform(root, &root_transform);
            node.update_bounds(root);
            // TODO!: visualize final bounds
            scene.bounds = scene.bounds.union(&node.bounds);
        }

        scene
    }

    // pub fn update_transforms(&self) {
    //     let root_transform = Matrix4::identity();
    //     // create stack with root nodes
    //     let mut stack = self.nodes.iter().map(|node|)

    // }

    // TODO: flatten draw call hierarchy (global Vec<SPrimitive>?)
    pub fn draw(&mut self, root: &mut Root, camera: &CameraControls) {
        for node_id in &self.nodes {
            let mut node = root.nodes[*node_id].borrow_mut();
            node.draw(root, camera);
        }
    }

    // pub fn walk_nodes<F>(&self, callback: F)
    //     where F: Fn(&Node)
    // {
    //     for node in &self.nodes {
    //         callback(node);
    //         // let mut current_child = node;
    //         node.walk_children(callback);

    //     }
    // }
}
