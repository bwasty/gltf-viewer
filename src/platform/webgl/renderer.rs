use std::collections::HashMap;
use std::f32::consts::PI;
use std::rc::Rc;

use gltf;
use gltf::json::texture::MinFilter;

use web_sys::{WebGlBuffer,WebGlTexture,WebGlVertexArrayObject};
use web_sys::WebGl2RenderingContext as GL;

use log::{error, warn, info};

use crate::controls::{OrbitControls, ScreenPosition, ScreenSize, NavState};
use crate::controls::CameraMovement::*;
use crate::render::{Root,Scene};
use crate::viewer::{GltfViewer};

use crate::platform::{ShaderInfo};


pub struct GltfViewerRenderer {
    pub gl: Rc<GL>,
    pub size: ScreenSize,
    pub shaders: Vec<ShaderInfo>,
    pub buffers: Vec<Rc<WebGlBuffer>>,
    pub textures: Vec<Rc<WebGlTexture>>,
    pub vaos: Vec<Rc<WebGlVertexArrayObject>>,
}

impl GltfViewerRenderer {
    pub fn new(gl: Rc<GL>, width: u32, height: u32) -> Self {
        Self {
            gl,
            size: ScreenSize { width: width as f32, height: height as f32 },
            shaders: Vec::new(),
            buffers: Vec::new(),
            textures: Vec::new(),
            vaos: Vec::new(),
        }
    }

    // prepare gl context for loading
    pub fn init_viewer_gl_context(&self, headless: bool, visible: bool)     {
        if headless || !visible {
            // transparent background for screenshots
            self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        }
        else {
            self.gl.clear_color(0.1, 0.2, 0.3, 1.0);
        }

        self.gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
        self.gl.enable(GL::DEPTH_TEST);

        // TODO: draw in wireframe
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
    }

    // Returns whether to keep running
    pub fn process_events(&mut self, orbit_controls: &mut OrbitControls) -> bool {
        process_events(
            orbit_controls,
            &mut self.size)
    }
    

    // draws next frame
    pub fn draw(&mut self, scene: &mut Scene, root: &mut Root, orbit_controls: &OrbitControls) {
        // render
        unsafe {
            self.gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

            let cam_params = orbit_controls.camera_params();
            scene.draw(root, &cam_params, self);
        }
    }

    // screenshot functions (only for gl platform atm)
    pub fn screenshot(&mut self, scene: &mut Scene, root: &mut Root, orbit_controls: &OrbitControls, filename: &str) {
        panic!("not implemented")
    }
    pub fn multiscreenshot(&mut self, scene: &mut Scene, root: &mut Root, orbit_controls: &mut OrbitControls, filename: &str, count: u32) {
        let min_angle : f32 = 0.0 ;
        let max_angle : f32 =  2.0 * PI ;
        let increment_angle : f32 = ((max_angle - min_angle)/(count as f32)) as f32;
        let suffix_length = count.to_string().len();
        for i in 1..=count {
            orbit_controls.rotate_object(increment_angle);
            let dot = filename.rfind('.').unwrap_or_else(|| filename.len());
            let mut actual_name = filename.to_string();
            actual_name.insert_str(dot, &format!("_{:0suffix_length$}", i, suffix_length = suffix_length));
            self.screenshot(scene, root, orbit_controls, &actual_name[..]);
        }
    }
}


// input / event loop

fn process_events(
    mut orbit_controls: &mut OrbitControls,
    size: &mut ScreenSize) -> bool
{
    let mut keep_running = true;

    // TODO events

    keep_running
}
