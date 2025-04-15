// TODO!: remove when done adapting
#![allow(unused_variables)]
#![allow(dead_code)]

use std::f32::consts::PI;

#[derive(Copy, Clone)]
pub struct CameraOptions {
    pub index: i32,
    // TODO!: math types
    // pub position: Option<Vector3>,
    // pub target: Option<Vector3>,
    // pub fovy: Deg<f32>,
    pub straight: bool,
}

pub struct GltfViewer {}

impl GltfViewer {
    pub fn new(
        source: &str,
        width: u32,
        height: u32,
        headless: bool,
        visible: bool,
        camera_options: CameraOptions,
        scene_index: usize,
    ) -> GltfViewer {
        // TODO!!: check whole old implementation thoroughly for param handling...
        let /*mut*/ viewer = GltfViewer {};
        viewer
    }

    /// determine "nice" camera perspective from bounding box. Inspired by donmccurdy/three-gltf-viewer
    fn set_camera_from_bounds(&mut self, straight: bool) {
        // TODO!!: re-implement?
        unimplemented!()
    }

    pub fn screenshot(&mut self, filename: &str) {
        // TODO!!: re-implement
    }

    pub fn multiscreenshot(&mut self, filename: &str, count: u32) {
        let min_angle: f32 = 0.0;
        let max_angle: f32 = 2.0 * PI;
        let increment_angle: f32 = ((max_angle - min_angle) / (count as f32)) as f32;
        let suffix_length = count.to_string().len();
        for i in 1..=count {
            // TODO!: adapt to bevy
            // self.orbit_controls.rotate_object(increment_angle);
            let dot = filename.rfind('.').unwrap_or_else(|| filename.len());
            let mut actual_name = filename.to_string();
            actual_name.insert_str(
                dot,
                &format!("_{:0suffix_length$}", i, suffix_length = suffix_length),
            );
            self.screenshot(&actual_name[..]);
        }
    }
}
