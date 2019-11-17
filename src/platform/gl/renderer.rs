use std::collections::HashMap;
use std::f32::consts::PI;
use std::os::raw::c_void;

use ::gl;

use glutin;
use glutin::{
    Api,
    MouseScrollDelta,
    MouseButton,
    GlContext,
    GlRequest,
    GlProfile,
    VirtualKeyCode,
    WindowEvent,
};
use glutin::dpi::PhysicalSize;
use glutin::ElementState::*;

use gltf;
use gltf::json::texture::MinFilter;

use image::{DynamicImage};

use log::{error, warn, info};

use crate::controls::{OrbitControls, ScreenPosition, ScreenSize, NavState};
use crate::controls::CameraMovement::*;
use crate::render::{Root,Scene};
use crate::viewer::{GltfViewer};

use crate::platform::{Framebuffer};

pub struct GltfViewerRenderer {
    pub size: ScreenSize,
    pub dpi_factor: f64,

    events_loop: Option<glutin::EventsLoop>,
    gl_window: Option<glutin::GlWindow>,

    pub uniform_location_map: HashMap<u32, HashMap<&'static str, i32>>,
}

impl GltfViewerRenderer {
    pub fn new(width: u32, height: u32, headless: bool,visible: bool) -> Self {
        let gl_request = GlRequest::Specific(Api::OpenGl, (3, 3));
        let gl_profile = GlProfile::Core;
        let (events_loop, gl_window, dpi_factor, inner_size) =
            if headless {
                let headless_context = glutin::HeadlessRendererBuilder::new(width, height)
                    // .with_gl(gl_request)
                    // .with_gl_profile(gl_profile)
                    .build()
                    .unwrap();
                unsafe { headless_context.make_current().unwrap() }
                gl::load_with(|symbol| headless_context.get_proc_address(symbol) as *const _);
                let framebuffer = Framebuffer::new(width, height);
                framebuffer.bind();
                unsafe { gl::Viewport(0, 0, width as i32, height as i32); }

                (None, None, 1.0, PhysicalSize::new(width as f64, height as f64)) // TODO: real height (retina? (should be the same as PhysicalSize when headless?))
            }
            else {
                // glutin: initialize and configure
                let events_loop = glutin::EventsLoop::new();
                let window_size = glutin::dpi::LogicalSize::new(width as f64, height as f64);

                // TODO?: hints for 4.1, core profile, forward compat
                let window = glutin::WindowBuilder::new()
                        .with_title("gltf-viewer")
                        .with_dimensions(window_size)
                        .with_visibility(visible);

                let context = glutin::ContextBuilder::new()
                    .with_gl(gl_request)
                    .with_gl_profile(gl_profile)
                    .with_vsync(true);
                let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

                // Real dimensions might be much higher on High-DPI displays
                let dpi_factor = gl_window.get_hidpi_factor();
                let inner_size = gl_window.get_inner_size().unwrap().to_physical(dpi_factor);

                unsafe { gl_window.make_current().unwrap(); }

                // gl: load all OpenGL function pointers
                gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

                (Some(events_loop), Some(gl_window), dpi_factor, inner_size)
            };
        

        Self {
            size: ScreenSize::new(inner_size.width, inner_size.height),
            dpi_factor,
            events_loop,
            gl_window,
            uniform_location_map: HashMap::new()
        }
    }

    // prepare gl context for loading
    pub fn init_viewer_gl_context(&self, headless: bool,visible: bool)     {
        unsafe {
            crate::platform::gl::utils::print_context_info();

            gl::ClearColor(0.0, 1.0, 0.0, 1.0); // green for debugging
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            if headless || !visible {
                // transparent background for screenshots
                gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            }
            else {
                gl::ClearColor(0.1, 0.2, 0.3, 1.0);
            }

            gl::Enable(gl::DEPTH_TEST);

            // TODO: keyboard switch?
            // draw in wireframe
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        };
    }

    // Returns whether to keep running
    pub fn process_events(&mut self, orbit_controls: &mut OrbitControls) -> bool {
        process_events(
            &mut self.events_loop.as_mut().unwrap(),
            self.gl_window.as_mut().unwrap(),
            orbit_controls,
            &mut self.dpi_factor,
            &mut self.size)
    }

    // draws next frame
    pub fn draw(&mut self, scene: &mut Scene, root: &mut Root, orbit_controls: &OrbitControls) {
        // render
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let cam_params = orbit_controls.camera_params();
            scene.draw(root, &cam_params, self);
        }

        // swap buffers
        self.gl_window.as_ref().unwrap().swap_buffers().unwrap();
    }

    // screenshot functions (only for gl platform atm)
    pub fn screenshot(&mut self, scene: &mut Scene, root: &mut Root, orbit_controls: &OrbitControls, filename: &str) {
        self.draw(scene, root, orbit_controls);

        let mut img = DynamicImage::new_rgba8(self.size.width as u32, self.size.height as u32);
        unsafe {
            let pixels = img.as_mut_rgba8().unwrap();
            gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
            gl::ReadPixels(0, 0, self.size.width as i32, self.size.height as i32, gl::RGBA,
                gl::UNSIGNED_BYTE, pixels.as_mut_ptr() as *mut c_void);

            // check error
            crate::platform::gl::utils::gl_check_error();
        }

        let img = img.flipv();
        if let Err(err) = img.save(filename) {
            error!("{}", err);
        }
        else {
            println!("Saved {}x{} screenshot to {}", self.size.width, self.size.height, filename);
        }
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

#[allow(clippy::too_many_arguments)]
fn process_events(
    events_loop: &mut glutin::EventsLoop,
    gl_window: &glutin::GlWindow,
    mut orbit_controls: &mut OrbitControls,
    dpi_factor: &mut f64,
    size: &mut ScreenSize) -> bool
{
    let mut keep_running = true;
    #[allow(clippy::single_match)]
    events_loop.poll_events(|event| {
        match event {
            glutin::Event::WindowEvent{ event, .. } => match event {
                WindowEvent::CloseRequested => {
                    keep_running = false;
                },
                WindowEvent::Destroyed => {
                    // Log and exit?
                    panic!("WindowEvent::Destroyed, unimplemented.");
                },
                WindowEvent::Resized(logical) => {
                    let ph = logical.to_physical(*dpi_factor);
                    gl_window.resize(ph);

                    // This doesn't seem to be needed on macOS but linux X11, Wayland and Windows
                    // do need it.
                    unsafe { gl::Viewport(0, 0, ph.width as i32, ph.height as i32); }

                    *size = ScreenSize::new(ph.width, ph.height);
                    orbit_controls.camera.update_aspect_ratio((ph.width / ph.height) as f32);
                    orbit_controls.screen_size = ScreenSize::new(ph.width, ph.height);
                },
                WindowEvent::HiDpiFactorChanged(f) => {
                    *dpi_factor = f;
                },
                WindowEvent::DroppedFile(_path_buf) => {
                    // TODO: drag file in
                }
                WindowEvent::MouseInput { button, state: Pressed, ..} => {
                    match button {
                        MouseButton::Left => {
                            orbit_controls.state = NavState::Rotating;
                        },
                        MouseButton::Right => {
                            orbit_controls.state = NavState::Panning;
                        },
                        _ => ()
                    }
                },
                WindowEvent::MouseInput { button, state: Released, ..} => {
                    match (button, orbit_controls.state.clone()) {
                        (MouseButton::Left, NavState::Rotating) | (MouseButton::Right, NavState::Panning) => {
                            orbit_controls.state = NavState::None;
                            orbit_controls.handle_mouse_up();
                        },
                        _ => ()
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let ph = position.to_physical(*dpi_factor);
                    orbit_controls.handle_mouse_move(ScreenPosition::new(ph.x,ph.y))
                },
                WindowEvent::MouseWheel { delta: MouseScrollDelta::PixelDelta(logical), .. } => {
                    let ph = logical.to_physical(*dpi_factor);
                    orbit_controls.process_mouse_scroll(ph.y as f32);
                }
                WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_rows, lines), .. } => {
                    orbit_controls.process_mouse_scroll(lines * 3.0);
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    keep_running = process_input(input, &mut orbit_controls);
                }
                _ => ()
            },
            _ => ()
        }
    });

    keep_running
}

fn process_input(input: glutin::KeyboardInput, controls: &mut OrbitControls) -> bool {
    let pressed = match input.state {
        Pressed => true,
        Released => false
    };
    if let Some(code) = input.virtual_keycode {
        match code {
            VirtualKeyCode::Escape if pressed => return false,
            VirtualKeyCode::W | VirtualKeyCode::Up    => controls.process_keyboard(FORWARD, pressed),
            VirtualKeyCode::S | VirtualKeyCode::Down  => controls.process_keyboard(BACKWARD, pressed),
            VirtualKeyCode::A | VirtualKeyCode::Left  => controls.process_keyboard(LEFT, pressed),
            VirtualKeyCode::D | VirtualKeyCode::Right => controls.process_keyboard(RIGHT, pressed),
            _ => ()
        }
    }
    true
}
