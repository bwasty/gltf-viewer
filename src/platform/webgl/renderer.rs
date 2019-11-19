use std::collections::HashMap;
use std::f32::consts::PI;
use std::cell::RefCell;
use std::rc::Rc;

use gltf;
use gltf::json::texture::MinFilter;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::*;
use web_sys::WebGl2RenderingContext as GL;

use crate::{debug};
use crate::controls::{OrbitControls, ScreenPosition, ScreenSize, NavState};
use crate::controls::CameraMovement::*;
use crate::render::{Root,Scene};
use crate::viewer::{GltfViewer};

use crate::platform::{ShaderInfo};


pub struct GltfViewerRenderer {
    pub canvas: Rc<HtmlCanvasElement>,
    pub gl: Rc<GL>,
    pub size: ScreenSize,
    pub shaders: Vec<ShaderInfo>,
    pub buffers: Vec<Rc<WebGlBuffer>>,
    pub textures: Vec<Rc<WebGlTexture>>,
    pub vaos: Vec<Rc<WebGlVertexArrayObject>>,
    pub messages: Rc<RefCell<Vec<GltfViewerRendererMsg>>>,
}


pub enum GltfViewerRendererMsg {
    MouseState(NavState),
    MouseMove(ScreenPosition),
    // Resized(ScreenSize),
    // Scrolled?
}

impl GltfViewerRenderer {
    pub fn new(canvas: Rc<HtmlCanvasElement>, gl: Rc<GL>, width: u32, height: u32) -> Self {
        Self {
            canvas,
            gl,
            size: ScreenSize { width: width as f32, height: height as f32 },
            shaders: Vec::new(),
            buffers: Vec::new(),
            textures: Vec::new(),
            vaos: Vec::new(),
            messages: Rc::new(RefCell::new(Vec::new())),
        }
    }

    // prepare gl context for loading
    pub fn init_viewer_gl_context(&self, headless: bool, visible: bool) {
        // init gl
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

        // bind events
        self.bind_mouse_down_event();
        self.bind_mouse_up_event();
        self.bind_mouse_move_event();
        self.bind_prevent_context_menu();
    }

    // Returns whether to keep running
    pub fn process_events(&mut self, orbit_controls: &mut OrbitControls) -> bool {
        let mut keep_running = true;
        let mut messages = self.messages.borrow_mut();

        // check events
        for msg in messages.iter() {
            match msg {
                GltfViewerRendererMsg::MouseState(nav_state) => orbit_controls.state = nav_state.clone(),
                GltfViewerRendererMsg::MouseMove(pos) => orbit_controls.handle_mouse_move(pos.clone()),
            }
        }

        // empty events
        messages.clear();

        keep_running
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


    // event bindings

    fn bind_mouse_down_event(&self) {
        let messages = self.messages.clone();
        let handler = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let state = match event.which() {
                1 => NavState::Rotating,
                3 => NavState::Panning,
                _ => NavState::None,
            };
            messages.borrow_mut().push(GltfViewerRendererMsg::MouseState(state));
        }) as Box<dyn FnMut(_)>);
        self.canvas.add_event_listener_with_callback("mousedown", handler.as_ref().unchecked_ref()).unwrap();
        handler.forget();
    }

    fn bind_mouse_up_event(&self) {
        let messages = self.messages.clone();
        let handler = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            messages.borrow_mut().push(GltfViewerRendererMsg::MouseState(NavState::None));
        }) as Box<dyn FnMut(_)>);
        self.canvas.add_event_listener_with_callback("mouseup", handler.as_ref().unchecked_ref()).unwrap();
        handler.forget();
    }

    fn bind_mouse_move_event(&self) {
        let messages = self.messages.clone();
        let handler = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let x = event.client_x();
            let y = event.client_y();
            messages.borrow_mut().push(GltfViewerRendererMsg::MouseMove(ScreenPosition::new(x as f64, y as f64)));
        }) as Box<dyn FnMut(_)>);
        self.canvas.add_event_listener_with_callback("mousemove", handler.as_ref().unchecked_ref()).unwrap();
        handler.forget();
    }

    fn bind_prevent_context_menu(&self) {
        let messages = self.messages.clone();
        let handler = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            event.prevent_default();
        }) as Box<dyn FnMut(_)>);
        self.canvas.add_event_listener_with_callback("contextmenu", handler.as_ref().unchecked_ref()).unwrap();
        handler.forget();
    }
}
