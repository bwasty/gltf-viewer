#![allow(dead_code)]
// #![allow(unused_features)]
// #![feature(test)]
#[macro_use] extern crate clap;
extern crate cgmath;
use cgmath::{Matrix4, Point3, Deg, perspective};
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
extern crate image;
use image::{DynamicImage, ImageFormat};

extern crate futures;

use clap::{Arg, App, AppSettings};

use std::fs::File;
use std::time::Instant;
use std::os::raw::c_void;

#[macro_use]extern crate log;
extern crate simplelog;
use simplelog::{TermLogger, LogLevelFilter, Config};

mod shader;
use shader::Shader;
mod camera;
use camera::Camera;
use camera::CameraMovement::*;
mod framebuffer;
use framebuffer::Framebuffer;
mod macros;
// TODO!!: adapt Source...
// mod http_source;
// use http_source::HttpSource;
mod utils;
use utils::{print_elapsed, FrameTimer, gl_check_error};

mod render;
use render::*;

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
            .short("-v")
            .help("Enable verbose logging."))
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

    let log_level = if args.is_present("verbose") { LogLevelFilter::Info } else { LogLevelFilter::Warn };
    let _ = TermLogger::init(log_level, Config { time: None, target: None, ..Config::default() });

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

struct GltfViewer {
    width: u32,
    height: u32,

    camera: Camera,
    first_mouse: bool,
    last_x: f32,
    last_y: f32,
    events_loop: Option<glutin::EventsLoop>,
    gl_window: Option<glutin::GlWindow>,

    shader: Shader,
    loc_projection: i32,
    loc_view: i32,

    scene: Scene,

    delta_time: f64, // seconds
    last_frame: Instant,

    render_timer: FrameTimer,
}

impl GltfViewer {
    pub fn new(source: &str, width: u32, height: u32, headless: bool, visible: bool) -> GltfViewer {
        let camera = Camera {
            // TODO!: position.z - bounding box length
            position: Point3::new(0.0, 0.0, 2.0),
            zoom: 60.0,
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

        let (shader, loc_projection, loc_view) = unsafe {
            gl::ClearColor(0.0, 1.0, 0.0, 1.0); // green for debugging
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::Enable(gl::DEPTH_TEST);

            // TODO!!!: switch again before release!
            // let mut shader = Shader::from_source(
            //     include_str!("shaders/simple.vs"),
            //     include_str!("shaders/simple.fs"));

            // NOTE: shader debug version
            let mut shader = Shader::new(
                "src/shaders/simple.vs",
                "src/shaders/simple.fs");

            shader.use_program();
            let loc_projection = shader.uniform_location("projection");
            let loc_view = shader.uniform_location("view");

            // TODO: keyboard switch?
            // draw in wireframe
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

            (shader, loc_projection, loc_view)
        };

        GltfViewer {
            width,
            height,

            camera,
            first_mouse, last_x, last_y,

            events_loop,
            gl_window,

            shader, loc_projection, loc_view,

            scene: Self::load(source),

            delta_time: 0.0, // seconds
            last_frame: Instant::now(),

            render_timer: FrameTimer::new("rendering", 300),
        }
    }

    pub fn load(source: &str) -> Scene {
        let mut start_time = Instant::now();
        // TODO!!: http source
        // let gltf =
        //     if source.starts_with("http") {
        //         unimplemented!()
        //         // let http_source = HttpSource::new(source);
        //         // let import = gltf::Import::custom(http_source, Default::default());
        //         // let gltf = import_gltf(import);
        //         // println!(); // to end the "progress dots"
        //         // gltf
        //     }
        //     else {
        let mut importer = gltf_importer::Importer::new(source);
        let gltf = match importer.import() {
            Ok(gltf) => gltf,
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
        let scene = Scene::from_gltf(gltf.scenes().nth(0).unwrap());
        print_elapsed(&format!("Loaded scene with {:<2} nodes {:<2} meshes in",
                gltf.nodes().count(), scene.meshes.len()), &start_time);

        scene
    }

    pub fn start_render_loop(&mut self) {
        loop {
            // per-frame time logic
            // NOTE: Deliberately ignoring the seconds of `elapsed()`
            self.delta_time = (self.last_frame.elapsed().subsec_nanos() as f64) / 1_000_000_000.0;
            self.last_frame = Instant::now();

            // events
            let keep_running = process_events(
                &mut self.events_loop.as_mut().unwrap(), &self.gl_window.as_mut().unwrap(),
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

            self.shader.use_program();

            // view/projection transformations
            // TODO!: only re-compute/set perspective on Zoom changes (also view?)
            let projection: Matrix4<f32> = perspective(Deg(self.camera.zoom), self.width as f32 / self.height as f32, 0.01, 1000.0);
            let view = self.camera.get_view_matrix();
            self.shader.set_mat4(self.loc_projection, &projection);
            self.shader.set_mat4(self.loc_view, &view);

            self.scene.draw(&mut self.shader);

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
            gl_check_error!();
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

// TODO!!:?
// fn import_gltf<S: gltf::import::Source>(import: gltf::Import<S>) -> gltf::Gltf {
//     match import.sync() {
//         Ok(gltf) => gltf,
//         Err(err) => {
//             error!("glTF import failed: {:?}", err);
//             std::process::exit(1);
//         }
//     }
// }

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
                },
                WindowEvent::DroppedFile(_path_buf) => (), // TODO!: drag file in
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
