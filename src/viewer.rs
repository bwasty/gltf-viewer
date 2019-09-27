use std::f32::consts::PI;
use std::os::raw::c_void;
use std::path::Path;
use std::process;
use std::time::Instant;

use cgmath::{ Deg, Point3 };
use collision::Aabb;
use gl;
use gltf;
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

use image::{DynamicImage};
use log::{error, warn, info};

use crate::controls::{OrbitControls, NavState};
use crate::controls::CameraMovement::*;
use crate::framebuffer::Framebuffer;
use crate::importdata::ImportData;
use crate::render::*;
use crate::render::math::*;
use crate::utils::{print_elapsed, FrameTimer, gl_check_error, print_context_info};

// TODO!: complete and pass through draw calls? or get rid of multiple shaders?
// How about state ordering anyway?
// struct DrawState {
//     current_shader: ShaderFlags,
//     back_face_culling_enabled: bool
// }

#[derive(Copy, Clone)]
pub struct CameraOptions {
    pub index: i32,
    pub position: Option<Vector3>,
    pub target: Option<Vector3>,
    pub fovy: Deg<f32>,
    pub straight: bool,
}

pub struct GltfViewer {
    size: PhysicalSize,
    dpi_factor: f64,

    orbit_controls: OrbitControls,
    events_loop: Option<glutin::EventsLoop>,
    gl_window: Option<glutin::GlWindow>,

    // TODO!: get rid of scene?
    root: Root,
    scene: Scene,

    delta_time: f64, // seconds
    last_frame: Instant,

    render_timer: FrameTimer,
}

/// Note about `headless` and `visible`: True headless rendering doesn't work on
/// all operating systems, but an invisible window usually works
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
        
        let mut orbit_controls = OrbitControls::new(
            Point3::new(0.0, 0.0, 2.0),
            inner_size);
        orbit_controls.camera = Camera::default();
        orbit_controls.camera.fovy = camera_options.fovy;
        orbit_controls.camera.update_aspect_ratio(inner_size.width as f32 / inner_size.height as f32); // updates projection matrix

        unsafe {
            print_context_info();

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

        let (root, scene) = Self::load(source, scene_index);
        let mut viewer = GltfViewer {
            size: inner_size,
            dpi_factor,

            orbit_controls,

            events_loop,
            gl_window,

            root,
            scene,

            delta_time: 0.0, // seconds
            last_frame: Instant::now(),

            render_timer: FrameTimer::new("rendering", 300),
        };
        unsafe { gl_check_error!(); };

        if camera_options.index != 0 && camera_options.index >= viewer.root.camera_nodes.len() as i32 {
            error!("No camera with index {} found in glTF file (max: {})",
                camera_options.index, viewer.root.camera_nodes.len() as i32 - 1);
            process::exit(2)
        }
        if !viewer.root.camera_nodes.is_empty() && camera_options.index != -1 {
            let cam_node = &viewer.root.get_camera_node(camera_options.index as usize);
            let cam_node_info = format!("{} ({:?})", cam_node.index, cam_node.name);
            let cam = cam_node.camera.as_ref().unwrap();
            info!("Using camera {} on node {}", cam.description(), cam_node_info);
            viewer.orbit_controls.set_camera(cam, &cam_node.final_transform);

            if camera_options.position.is_some() || camera_options.target.is_some() {
                warn!("Ignoring --cam-pos / --cam-target since --cam-index is given.")
            }
        } else {
            info!("Determining camera view from bounding box");
            viewer.set_camera_from_bounds(camera_options.straight);

            if let Some(p) = camera_options.position {
                viewer.orbit_controls.position = Point3::from_vec(p)
            }
            if let Some(target) = camera_options.target {
                viewer.orbit_controls.target = Point3::from_vec(target)
            }
        }

        viewer
    }

    pub fn load(source: &str, scene_index: usize) -> (Root, Scene) {
        let mut start_time = Instant::now();
        // TODO!: http source
        // let gltf =
        if source.starts_with("http") {
            panic!("not implemented: HTTP support temporarily removed.")
            // let http_source = HttpSource::new(source);
            // let import = gltf::Import::custom(http_source, Default::default());
            // let gltf = import_gltf(import);
            // println!(); // to end the "progress dots"
            // gltf
        }
        //     else {
        let (doc, buffers, images) = match gltf::import(source) {
            Ok(tuple) => tuple,
            Err(err) => {
                error!("glTF import failed: {:?}", err);
                if let gltf::Error::Io(_) = err {
                    error!("Hint: Are the .bin file(s) referenced by the .gltf file available?")
                }
                process::exit(1)
            },
        };
        let imp = ImportData { doc, buffers, images };

        print_elapsed("Imported glTF in ", start_time);
        start_time = Instant::now();

        // load first scene
        if scene_index >= imp.doc.scenes().len() {
            error!("Scene index too high - file has only {} scene(s)", imp.doc.scenes().len());
            process::exit(3)
        }
        let base_path = Path::new(source);
        let mut root = Root::from_gltf(&imp, base_path);
        let scene = Scene::from_gltf(&imp.doc.scenes().nth(scene_index).unwrap(), &mut root);
        print_elapsed(&format!("Loaded scene with {} nodes, {} meshes in ",
                imp.doc.nodes().count(), imp.doc.meshes().len()), start_time);

        (root, scene)
    }

    /// determine "nice" camera perspective from bounding box. Inspired by donmccurdy/three-gltf-viewer
    fn set_camera_from_bounds(&mut self, straight: bool) {
        let bounds = &self.scene.bounds;
        let size = (bounds.max - bounds.min).magnitude();
        let center = bounds.center();

        // TODO: x,y addition optional
        let cam_pos = if straight {
            Point3::new(
                center.x,
                center.y,
                center.z + size * 0.75,
            )
        } else {
            Point3::new(
                center.x + size / 2.0,
                center.y + size / 5.0,
                center.z + size / 2.0,
            )
        };

        self.orbit_controls.position = cam_pos;
        self.orbit_controls.target = center;
        self.orbit_controls.camera.znear = size / 100.0;
        self.orbit_controls.camera.zfar = Some(size * 20.0);
        self.orbit_controls.camera.update_projection_matrix();
    }

    pub fn start_render_loop(&mut self) {
        loop {
            // per-frame time logic
            // NOTE: Deliberately ignoring the seconds of `elapsed()`
            self.delta_time = f64::from(self.last_frame.elapsed().subsec_nanos()) / 1_000_000_000.0;
            self.last_frame = Instant::now();

            // events
            let keep_running = process_events(
                &mut self.events_loop.as_mut().unwrap(),
                self.gl_window.as_mut().unwrap(),
                &mut self.orbit_controls,
                &mut self.dpi_factor,
                &mut self.size);
            if !keep_running {
                unsafe { gl_check_error!(); } // final error check so errors don't go unnoticed
                break
            }

            self.orbit_controls.frame_update(self.delta_time); // keyboard navigation

            self.draw();

            self.gl_window.as_ref().unwrap().swap_buffers().unwrap();
        }
    }

    // Returns whether to keep running
    pub fn draw(&mut self) {
        // render
        unsafe {
            self.render_timer.start();

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let cam_params = self.orbit_controls.camera_params();
            self.scene.draw(&mut self.root, &cam_params);

            self.render_timer.end();
        }
    }

    pub fn screenshot(&mut self, filename: &str) {
        self.draw();

        let mut img = DynamicImage::new_rgba8(self.size.width as u32, self.size.height as u32);
        unsafe {
            let pixels = img.as_mut_rgba8().unwrap();
            gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
            gl::ReadPixels(0, 0, self.size.width as i32, self.size.height as i32, gl::RGBA,
                gl::UNSIGNED_BYTE, pixels.as_mut_ptr() as *mut c_void);
            gl_check_error!();
        }

        let img = img.flipv();
        if let Err(err) = img.save(filename) {
            error!("{}", err);
        }
        else {
            println!("Saved {}x{} screenshot to {}", self.size.width, self.size.height, filename);
        }
    }
    pub fn multiscreenshot(&mut self, filename: &str, count: u32) {
        let min_angle : f32 = 0.0 ;
        let max_angle : f32 =  2.0 * PI ;
        let increment_angle : f32 = ((max_angle - min_angle)/(count as f32)) as f32;
        let suffix_length = count.to_string().len();
        for i in 1..=count {
            self.orbit_controls.rotate_object(increment_angle);
            let dot = filename.rfind('.').unwrap_or_else(|| filename.len());
            let mut actual_name = filename.to_string();
            actual_name.insert_str(dot, &format!("_{:0suffix_length$}", i, suffix_length = suffix_length));
            self.screenshot(&actual_name[..]);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn process_events(
    events_loop: &mut glutin::EventsLoop,
    gl_window: &glutin::GlWindow,
    mut orbit_controls: &mut OrbitControls,
    dpi_factor: &mut f64,
    size: &mut PhysicalSize) -> bool
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

                    *size = ph;
                    orbit_controls.camera.update_aspect_ratio((ph.width / ph.height) as f32);
                    orbit_controls.screen_size = ph;
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
                    orbit_controls.handle_mouse_move(ph)
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
