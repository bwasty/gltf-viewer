use std::cell::RefCell;
use std::rc::Rc;
use cgmath::Deg;
use js_sys::{Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::viewer::{CameraOptions,GltfViewer};

#[wasm_bindgen]
pub struct GltfViewerApp {
    viewer: Rc<RefCell<GltfViewer>>,
    camera_options: Rc<CameraOptions>,
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
        let camera_options = Rc::new(CameraOptions {
            index: 0,
            position: None,
            target: None,
            fovy: Deg(75.0),
            straight: true,
        });

        let viewer = Rc::new(RefCell::new(GltfViewer::new_from_webgl(
            Rc::clone(&canvas),
            Rc::new(gl),
            canvas.client_width() as u32,
            canvas.client_height() as u32,
            false,
            true,
            camera_options.as_ref(),
        )));

        GltfViewerApp {
            viewer,
            camera_options,
        }
    }

    pub fn load_file(&mut self, js_file_name: JsValue, js_array_buffer: JsValue) {
        let uint8_view = Uint8Array::new(&js_array_buffer.dyn_into::<js_sys::ArrayBuffer>().expect("Expected buffer"));
        let mut bytes = vec![0; uint8_view.length() as usize];
        uint8_view.copy_to(&mut bytes[..]);
        
        let filename = String::from(&js_file_name.dyn_into::<js_sys::JsString>().expect("Expected filename")).to_string();
        
        self.viewer.borrow_mut().load_from_bytes(&bytes.as_slice(), &filename, 0, self.camera_options.as_ref());
    }

    pub fn run_frame(&mut self, delta: f64) {
        self.viewer.borrow_mut().render_frame(delta);
    }
}
