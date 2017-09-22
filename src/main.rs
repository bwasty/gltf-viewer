#![allow(dead_code)]
// #![allow(unused_features)]
// #![feature(test)]
#[macro_use] extern crate clap;
extern crate cgmath;
use cgmath::{ Point3 };
// use cgmath::prelude::*;

extern crate gl;

extern crate glutin;
use glutin::{
    CursorState,
    ElementState,
    MouseScrollDelta,
    GlContext,
    VirtualKeyCode,
    WindowEvent,
};

extern crate gltf;
extern crate gltf_importer;
use gltf_importer::Config;
use gltf_importer::config::ValidationStrategy;
extern crate gltf_utils;
extern crate image;
use image::{DynamicImage, ImageFormat};

#[macro_use]
extern crate bitflags;

use clap::{Arg, App, AppSettings};

use std::fs::File;
use std::time::Instant;
use std::os::raw::c_void;
use std::path::Path;

#[macro_use]extern crate log;
extern crate simplelog;
use simplelog::{TermLogger, LogLevelFilter, Config as LogConfig};

mod shader;
mod camera;
use camera::Camera;
use camera::CameraMovement::*;
mod framebuffer;
use framebuffer::Framebuffer;
mod macros;
// TODO!: adapt Source...
// mod http_source;
// use http_source::HttpSource;
mod utils;
use utils::{print_elapsed, FrameTimer, gl_check_error};

mod render;
use render::*;
use render::math::*;

pub fn main() {
    let args = App::new("gltf-viewer")
        .version(crate_version!())
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::DeriveDisplayOrder)
        .arg(Arg::with_name("FILE/URL")
            .required(true)
            .takes_value(true)
            .help("glTF file name or URL"))
        .arg(Arg::with_name("screenshot")
            .long("screenshot")
            .short("s")
            .value_name("FILE")
            .help("Create screenshot (PNG)"))
        .arg(Arg::with_name("verbose")
            .long("verbose")
            .short("v")
            .multiple(true)
            .help("Enable verbose logging (log level INFO). Can be repeated multiple times to increase to log level DEBUG/TRACE)"))
        .arg(Arg::with_name("WIDTH")
            .long("width")
            .short("w")
            .default_value("800")
            .help("Width in pixels")
            .validator(|value| value.parse::<u32>().map(|_| ()).map_err(|err| err.to_string())))
        .arg(Arg::with_name("HEIGHT")
            .long("height")
            .short("h")
            .default_value("600")
            .help("Height in pixels")
            .validator(|value| value.parse::<u32>().map(|_| ()).map_err(|err| err.to_string())))
        .get_matches();
    let source = args.value_of("FILE/URL").unwrap();
    let width: u32 = args.value_of("WIDTH").unwrap().parse().unwrap();
    let height: u32 = args.value_of("HEIGHT").unwrap().parse().unwrap();

    let log_level = match args.occurrences_of("verbose") {
        0 => LogLevelFilter::Warn,
        1 => LogLevelFilter::Info,
        2 => LogLevelFilter::Debug,
        _ => LogLevelFilter::Trace
    };

    let _ = TermLogger::init(log_level, LogConfig { time: None, target: None, ..LogConfig::default() });

    // TODO!: headless rendering doesn't work (only clearcolor)
    let mut viewer = GltfViewer::new(source, width, height,
        // args.is_present("screenshot")
        false,
        !args.is_present("screenshot")
    );

    if args.is_present("screenshot") {
        let filename = args.value_of("screenshot").unwrap();
        if !filename.to_lowercase().ends_with(".png") {
            warn!("filename should end with .png");
        }
        viewer.screenshot(filename, width, height);
        return;
    }

    viewer.start_render_loop();
}

// TODO!: complete and pass through draw calls? or get rid of multiple shaders?
// How about state ordering anyway?
// struct DrawState {
//     current_shader: ShaderFlags,
// }

struct GltfViewer {
    width: u32,
    height: u32,

    camera: Camera,
    first_mouse: bool,
    last_x: f32,
    last_y: f32,
    events_loop: Option<glutin::EventsLoop>,
    gl_window: Option<glutin::GlWindow>,

    scene: Scene,

    delta_time: f64, // seconds
    last_frame: Instant,

    render_timer: FrameTimer,
}

// TODO!: move to separate file
impl GltfViewer {
    pub fn new(source: &str, width: u32, height: u32, headless: bool, visible: bool) -> GltfViewer {
        let camera = Camera {
            position: Point3::new(0.0, 0.0, 2.0),
            zoom: 60.0,
            aspect_ratio: width as f32 / height as f32,
            ..Camera::default()
        };

        let first_mouse = true;
        let last_x: f32 = width as f32 / 2.0;
        let last_y: f32 = height as f32 / 2.0;

        let (events_loop, gl_window) =
            if headless {
                let headless_context = glutin::HeadlessRendererBuilder::new(width, height).build().unwrap();
                unsafe { headless_context.make_current().unwrap() }
                gl::load_with(|symbol| headless_context.get_proc_address(symbol) as *const _);
                let framebuffer = Framebuffer::new(width, height);
                framebuffer.bind();

                (None, None)
            }
            else {
                // glutin: initialize and configure
                let events_loop = glutin::EventsLoop::new().into();

                // TODO?: hints for 4.1, core profile, forward compat
                let window = glutin::WindowBuilder::new()
                        .with_title("gltf-viewer")
                        .with_dimensions(width, height)
                        .with_visibility(visible);

                let context = glutin::ContextBuilder::new()
                    .with_vsync(true);
                let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

                unsafe { gl_window.make_current().unwrap(); }

                // TODO!: capturing - on click or uncapture somehow?
                // TODO!: find solution for macOS - see https://github.com/tomaka/glutin/issues/226
                #[cfg(target_os = "macos")]
                let _ = gl_window.set_cursor_state(CursorState::Hide);
                #[cfg(not(target_os = "macos"))]
                let _ = gl_window.set_cursor_state(CursorState::Grab);

                // gl: load all OpenGL function pointers
                gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

                (Some(events_loop), Some(gl_window))
            };

        unsafe {
            gl::ClearColor(0.0, 1.0, 0.0, 1.0); // green for debugging
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::Enable(gl::DEPTH_TEST);

            // TODO: keyboard switch?
            // draw in wireframe
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        };

        let mut viewer = GltfViewer {
            width,
            height,

            camera,
            first_mouse, last_x, last_y,

            events_loop,
            gl_window,

            scene: Self::load(source),

            delta_time: 0.0, // seconds
            last_frame: Instant::now(),

            render_timer: FrameTimer::new("rendering", 300),
        };
        unsafe { gl_check_error!(); }
        viewer.set_camera_from_bounds();
        viewer
    }

    pub fn load(source: &str) -> Scene {
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
        let config = Config { validation_strategy: ValidationStrategy::Complete };
        let (gltf, buffers) = match gltf_importer::import_with_config(source, config) {
            Ok((gltf, buffers)) => (gltf, buffers),
            Err(err) => {
                error!("glTF import failed: {}", err);
                std::process::exit(1);
            },
        };

        print_elapsed("Imported glTF in ", &start_time);
        start_time = Instant::now();

        // load first scene
        if gltf.scenes().len() > 1 {
            warn!("Found more than 1 scene, can only load first at the moment.")
        }
        let base_path = Path::new(source);
        let scene = Scene::from_gltf(gltf.scenes().nth(0).unwrap(), &buffers, base_path);
        print_elapsed(&format!("Loaded scene with {} nodes, {} meshes in ",
                gltf.nodes().count(), scene.meshes.len()), &start_time);

        scene
    }

    /// determine "nice" camera perspective from bounding box. Inspired by donmccurdy/three-gltf-viewer
    fn set_camera_from_bounds(&mut self) {
        // TODO!!: fix bounds/camera computation (many models NOT centered)
        let bounds = &self.scene.bounds;
        let size = bounds.size().magnitude();
        let center = bounds.center();

        // TODO!: move cam instead?
        let _obj_pos_modifier = -center;

        let _max_distance = size * 10.0;
        let cam_pos = Point3::new(
            center.x + size / 2.0,
            center.y + size / 5.0,
            center.z + size / 2.0,
        );
        let _near = size / 100.0;
        let _far = size * 100.0;

        self.camera.position = cam_pos;
        self.camera.center = Some(Point3::from_vec(center));
        // TODO!: set near, far, max_distance, obj_pos_modifier...
    }

    pub fn start_render_loop(&mut self) {
        loop {
            // per-frame time logic
            // NOTE: Deliberately ignoring the seconds of `elapsed()`
            self.delta_time = (self.last_frame.elapsed().subsec_nanos() as f64) / 1_000_000_000.0;
            self.last_frame = Instant::now();

            // events
            let keep_running = process_events(
                &mut self.events_loop.as_mut().unwrap(), self.gl_window.as_mut().unwrap(),
                &mut self.first_mouse, &mut self.last_x, &mut self.last_y,
                &mut self.camera, &mut self.width, &mut self.height);
            if !keep_running { break }

            self.camera.update(self.delta_time); // navigation

            self.draw();

            self.gl_window.as_ref().unwrap().swap_buffers().unwrap();
        }
    }

    // Returns whether to keep running
    pub fn draw(&mut self) {
        // render
        unsafe {
            self.render_timer.start();

            gl::ClearColor(0.1, 0.2, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            self.scene.draw(&self.camera);

            self.render_timer.end();
        }
    }

    pub fn screenshot(&mut self, filename: &str, _width: u32, _height: u32) {
        self.draw();

        // TODO!: headless case...
        let (width, height) = self.gl_window.as_ref().unwrap().get_inner_size_pixels().unwrap();
        let mut img = DynamicImage::new_rgb8(width, height);
        unsafe {
            let pixels = img.as_mut_rgb8().unwrap();
            gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
            gl::ReadPixels(0, 0, width as i32, height as i32, gl::RGB,
                gl::UNSIGNED_BYTE, pixels.as_mut_ptr() as *mut c_void);
        }

        let img = img.flipv();

        let mut file = File::create(filename).unwrap();
        if let Err(err) = img.save(&mut file, ImageFormat::PNG) {
            error!("{}", err);
        }
        else {
            println!("Saved {}x{} screenshot to {}", width, height, filename);
        }
    }
}

fn process_events(
    events_loop: &mut glutin::EventsLoop,
    gl_window: &glutin::GlWindow,
    first_mouse: &mut bool,
    last_x: &mut f32,
    last_y: &mut f32,
    mut camera: &mut Camera,
    width: &mut u32,
    height: &mut u32) -> bool
{
    let mut keep_running = true;
    events_loop.poll_events(|event| {
        match event {
            glutin::Event::WindowEvent{ event, .. } => match event {
                WindowEvent::Closed => keep_running = false,
                WindowEvent::Resized(w, h) => {
                    gl_window.resize(w, h);
                    *width = w;
                    *height = h;
                    // TODO!: update camera aspect?
                },
                WindowEvent::DroppedFile(_path_buf) => (), // TODO: drag file in
                WindowEvent::MouseMoved { position: (xpos, ypos), .. } => {
                    let (xpos, ypos) = (xpos as f32, ypos as f32);
                    if *first_mouse {
                        *last_x = xpos;
                        *last_y = ypos;
                        *first_mouse = false;
                    }

                    let xoffset = xpos - *last_x;
                    let yoffset = *last_y - ypos; // reversed since y-coordinates go from bottom to top

                    *last_x = xpos;
                    *last_y = ypos;

                    camera.process_mouse_movement(xoffset, yoffset, true);
                },
                WindowEvent::MouseWheel { delta: MouseScrollDelta::PixelDelta(_xoffset, yoffset), .. } => {
                    // TODO: need to handle LineDelta case too?
                    camera.process_mouse_scroll(yoffset);
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    keep_running = process_input(input, &mut camera);
                }
                _ => ()
            },
            _ => ()
        }
    });

    keep_running
}

fn process_input(input: glutin::KeyboardInput, camera: &mut Camera) -> bool {
    let pressed = match input.state {
        ElementState::Pressed => true,
        ElementState::Released => false
    };
    if let Some(code) = input.virtual_keycode {
        match code {
            VirtualKeyCode::Escape if pressed => return false,
            VirtualKeyCode::W => camera.process_keyboard(FORWARD, pressed),
            VirtualKeyCode::S => camera.process_keyboard(BACKWARD, pressed),
            VirtualKeyCode::A => camera.process_keyboard(LEFT, pressed),
            VirtualKeyCode::D => camera.process_keyboard(RIGHT, pressed),
            _ => ()
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_struct_sizes() {
        // run with `cargo test -- --nocapture`
        println!("Sizes in bytes:");
        println!("Scene:     {:>3}", std::mem::size_of::<Scene>());
        println!("Node:      {:>3}", std::mem::size_of::<Node>());
        println!("Mesh:      {:>3}", std::mem::size_of::<Mesh>());
        println!("Primitive: {:>3}", std::mem::size_of::<Primitive>());
        println!("Vertex:    {:>3}", std::mem::size_of::<Vertex>());
        println!();
        println!("Option<String>: {:>3}", std::mem::size_of::<Option<String>>());
        println!("String:         {:>3}", std::mem::size_of::<String>());
        println!("Vec<f32>:       {:>3}", std::mem::size_of::<Vec<f32>>());
        println!("Vec<Node>:      {:>3}", std::mem::size_of::<Vec<Node>>());
    }

//     extern crate test;
//     use self::test::Bencher;
//     #[bench]
//     fn bench_frame_timer(b: &mut Bencher) {
//         let mut timer = FrameTimer::new("Foobar", 60);
//         b.iter(|| {
//             for _ in 0..60 {
//                 timer.start();
//                 timer.end();
//             }
//         })
//     }
}
