use core::num::NonZeroU32;
use std::f32::consts::PI;
use std::ffi::CString;
use std::os::raw::c_void;
use std::path::Path;
use std::process;
use std::time::Instant;

use cgmath::{ Deg, Point3 };
use collision::Aabb;
use gl;
use gltf;
use glutin;
use glutin::context::{GlProfile, ContextApi, ContextAttributesBuilder, Version, NotCurrentGlContext};
use glutin::display::{DisplayApiPreference, GlDisplay};
use glutin::surface::GlSurface;
use winit::{
    event::{MouseScrollDelta, MouseButton, WindowEvent, ElementState::*},
    keyboard::{PhysicalKey, KeyCode},
    dpi::{PhysicalSize, LogicalSize},
    event_loop::{ActiveEventLoop, EventLoop},
};
use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};

use image::DynamicImage;
use log::{error, warn, info};

use crate::controls::{OrbitControls, NavState};
use crate::controls::CameraMovement::*;
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
    size: PhysicalSize<f64>,

    orbit_controls: OrbitControls,
    window: Option<winit::window::Window>,
    gl_context: Option<glutin::context::PossiblyCurrentContext>,
    gl_surface: Option<glutin::surface::Surface<glutin::surface::WindowSurface>>,

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
        event_loop: &mut EventLoop<()>,
        source: &str,
        width: u32,
        height: u32,
        headless: bool,
        visible: bool,
        camera_options: CameraOptions,
        scene_index: usize,
    ) -> GltfViewer {
        assert!(!headless, "Headless support is currently disabled");

        // glutin: initialize and configure
        let window_size = LogicalSize::new(width as f64, height as f64);

        // TODO?: hints for 4.1, core profile, forward compat
        let window_attributes = winit::window::Window::default_attributes()
            .with_title("gltf-viewer")
            .with_inner_size(window_size)
            .with_visible(visible);
        let window = event_loop.create_window(window_attributes).unwrap();

        let raw_window_handle = window.raw_window_handle();
        let inner_size = window.inner_size();

        let gl_display = unsafe { glutin::display::Display::new(window.raw_display_handle(), DisplayApiPreference::Egl) }.unwrap();

        let config_template = glutin::config::ConfigTemplateBuilder::new()
            .with_api(glutin::config::Api::OPENGL)
            .compatible_with_native_window(raw_window_handle)
            .build();
        let gl_config = unsafe { gl_display.find_configs(config_template) }.unwrap().next().unwrap();

        let surface_attributes = glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
            .build(raw_window_handle, NonZeroU32::new(inner_size.width).unwrap(), NonZeroU32::new(inner_size.height).unwrap());
        let gl_surface = unsafe { gl_display.create_window_surface(&gl_config, &surface_attributes) }.unwrap();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .with_profile(GlProfile::Core)
            .build(Some(raw_window_handle));
        let gl_context = unsafe { gl_display.create_context(&gl_config, &context_attributes) }.unwrap();
        let gl_context = gl_context.make_current(&gl_surface).unwrap();

        // gl: load all OpenGL function pointers
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(&symbol) as *const _
        });

        let window = Some(window);
        let gl_context = Some(gl_context);
        let gl_surface = Some(gl_surface);
        let inner_size = PhysicalSize::new(inner_size.width as f64, inner_size.height as f64);

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

            orbit_controls,

            window,
            gl_context,
            gl_surface,

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

    fn run_render_loop(&mut self) {
        // per-frame time logic
        // NOTE: Deliberately ignoring the seconds of `elapsed()`
        self.delta_time = f64::from(self.last_frame.elapsed().subsec_nanos()) / 1_000_000_000.0;
        self.last_frame = Instant::now();

        self.orbit_controls.frame_update(self.delta_time); // keyboard navigation

        self.draw();

        self.gl_surface.as_ref().unwrap().swap_buffers(self.gl_context.as_ref().unwrap()).unwrap();
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

impl winit::application::ApplicationHandler<()> for GltfViewer {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.run_render_loop();
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Destroyed => {
                // Log and exit?
                panic!("WindowEvent::Destroyed, unimplemented.");
            },
            WindowEvent::Resized(ph) => {

                // This doesn't seem to be needed on macOS but linux X11, Wayland and Windows
                // do need it.
                unsafe { gl::Viewport(0, 0, ph.width as i32, ph.height as i32); }

                self.size = PhysicalSize::new(ph.width as f64, ph.height as f64);
                self.orbit_controls.camera.update_aspect_ratio((self.size.width / self.size.height) as f32);
                self.orbit_controls.screen_size = self.size;
                self.window.as_ref().unwrap().request_redraw();
            },
            WindowEvent::DroppedFile(_path_buf) => {
                // TODO: drag file in
            }
            WindowEvent::MouseInput { button, state: Pressed, ..} => {
                match button {
                    MouseButton::Left => {
                        self.orbit_controls.state = NavState::Rotating;
                        self.window.as_ref().unwrap().request_redraw();
                    },
                    MouseButton::Right => {
                        self.orbit_controls.state = NavState::Panning;
                        self.window.as_ref().unwrap().request_redraw();
                    },
                    _ => ()
                }
            },
            WindowEvent::MouseInput { button, state: Released, ..} => {
                match (button, self.orbit_controls.state.clone()) {
                    (MouseButton::Left, NavState::Rotating) | (MouseButton::Right, NavState::Panning) => {
                        self.orbit_controls.state = NavState::None;
                        self.orbit_controls.handle_mouse_up();
                        self.window.as_ref().unwrap().request_redraw();
                    },
                    _ => ()
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.orbit_controls.handle_mouse_move(position);
                self.window.as_ref().unwrap().request_redraw();
            },
            WindowEvent::MouseWheel { delta: MouseScrollDelta::PixelDelta(ph), .. } => {
                self.orbit_controls.process_mouse_scroll(ph.y as f32);
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_rows, lines), .. } => {
                self.orbit_controls.process_mouse_scroll(lines * 3.0);
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::PanGesture { delta, .. } => {
                let mut delta = delta;
                delta.x /= 1024.;
                delta.y /= 1024.;
                self.orbit_controls.pan_both(delta);
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::PinchGesture { delta, .. } => {
                self.orbit_controls.process_mouse_scroll(1024. * delta as f32);
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RotationGesture { delta, .. } => {
                self.orbit_controls.rotate_object(-delta * std::f32::consts::PI / 180.);
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = match event.state {
                    Pressed => true,
                    Released => false
                };
                if let PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        KeyCode::Escape if pressed => event_loop.exit(),
                        KeyCode::KeyW | KeyCode::ArrowUp    => self.orbit_controls.process_keyboard(FORWARD, pressed),
                        KeyCode::KeyS | KeyCode::ArrowDown  => self.orbit_controls.process_keyboard(BACKWARD, pressed),
                        KeyCode::KeyA | KeyCode::ArrowLeft  => self.orbit_controls.process_keyboard(LEFT, pressed),
                        KeyCode::KeyD | KeyCode::ArrowRight => self.orbit_controls.process_keyboard(RIGHT, pressed),
                        _ => return
                    }
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => ()
        }
    }
}
