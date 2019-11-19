use std::cell::RefCell;
use std::rc::Rc;
use cgmath::Deg;
use web_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use viewer::{CameraOptions,GltfViewer};

/// lib exports for use as library

pub mod utils;
pub mod viewer;
// use crate::viewer::{GltfViewer, CameraOptions};

mod shader;
pub mod controls;
mod importdata;
// TODO!: adapt Source...
// mod http_source;
// use http_source::HttpSource;
mod platform;
pub mod render;


#[wasm_bindgen]
pub struct GltfViewerApp {
    viewer: Rc<RefCell<GltfViewer>>,
}

// Creates a canvas and initiats a request animation frame loop
#[wasm_bindgen]
impl GltfViewerApp {
    /// Constructs / initializes app classes
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_value: JsValue) -> GltfViewerApp {
        let canvas = Rc::new(canvas_value.dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Problem casting canvas"));
        let gl = canvas
            .get_context("webgl2")
            .expect("Problem getting canvas context")
            .expect("The canvas context is empty")
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .expect("Problem casting canvas context as web_sys::WebGl2RenderingContext");

        // camera configuration
        let camera_options = &CameraOptions {
            index: 0,
            position: None,
            target: None,
            fovy: Deg(75.0),
            straight: true,
        };

        let viewer = Rc::new(RefCell::new(GltfViewer::new_from_webgl(
            Rc::clone(&canvas),
            Rc::new(gl),
            canvas.client_width() as u32,
            canvas.client_height() as u32,
            false,
            true,
            camera_options,
        )));

        let source_bytes = &include_bytes!("../../../g/oly/static/Apple_Lisa.glb")[..];
        viewer.borrow_mut().load_from_bytes(source_bytes, &"apple lisa", 0, camera_options);

        GltfViewerApp {
            viewer,
        }
    }

    pub fn run_frame(&mut self, delta: f64) {
        self.viewer.borrow_mut().render_frame(delta);
    }
}
