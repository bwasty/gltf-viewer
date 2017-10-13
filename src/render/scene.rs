
use gltf;

use controls::CameraParams;
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
            let node = root.unsafe_get_node_mut(*node_id);
            node.update_transform(root, &root_transform);
            node.update_bounds(root);
            // TODO!: visualize final bounds
            scene.bounds = scene.bounds.union(&node.bounds);
        }

        scene
    }

    // TODO: flatten draw call hierarchy (global Vec<Primitive>?)
    pub fn draw(&mut self, root: &mut Root, cam_params: &CameraParams) {
        for node_id in &self.nodes {
            let node = root.unsafe_get_node_mut(*node_id);
            node.draw(root, cam_params);
        }
    }
}
