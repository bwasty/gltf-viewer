use std::path::Path;

use gltf;
use gltf_importer;

use camera::CameraControls;
use render::{Node, Root};
use render::math::*;

#[derive(Default)]
pub struct Scene {
    pub name: Option<String>,
    pub nodes: Vec<Node>, // TODO!!!: change to indices; all indices instead of or in addition to roots?
    pub bounds: Bounds,
}

impl Scene {
    pub fn from_gltf(g_scene: gltf::Scene, root: &mut Root, buffers: &gltf_importer::Buffers, base_path: &Path) -> Scene {
        let mut scene = Scene {
            name: g_scene.name().map(|s| s.to_owned()),
            ..Default::default()
        };
        scene.nodes = g_scene.nodes()
            .map(|g_node| Node::from_gltf(g_node, root, buffers, base_path))
            .collect();

        // propagate transforms
        let root_transform = Matrix4::identity();
        for node in &mut scene.nodes {
            node.update_transform(&root_transform);
            node.update_bounds();
            // TODO!: visualize final bounds
            scene.bounds = scene.bounds.union(&node.bounds);
        }

        scene
    }

    // TODO: flatten draw call hierarchy (global Vec<SPrimitive>?)
    pub fn draw(&mut self, camera: &CameraControls) {
        for node in &mut self.nodes {
            node.draw(camera);
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
