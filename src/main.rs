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
extern crate image;

extern crate futures;

use clap::{Arg, App};

use std::time::Instant;

mod shader;
use shader::Shader;
mod camera;
use camera::Camera;
use camera::CameraMovement::*;
mod framebuffer;
mod macros;
mod http_source;
use http_source::HttpSource;
mod utils;
use utils::{print_elapsed, FrameTimer};

mod render;
use render::*;

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

pub fn main() {
    let args = App::new("gltf-viewer")
        .version(crate_version!())
        .arg(Arg::with_name("FILE/URL")
            .required(true)
            .takes_value(true)
            .help("glTF file name or URL"))
        .arg(Arg::with_name("screenshot")
            .long("screenshot")
            .short("s")
            .help("Create screenshot (NOT WORKING YET)"))
        .get_matches();
    let source = args.value_of("FILE/URL").unwrap();
    let screenshot = args.is_present("screenshot");

    let mut viewer = GltfViewer::new(source);
    viewer.start_render_loop(screenshot);
}

struct GltfViewer {
    camera: Camera,
    first_mouse: bool,
    last_x: f32,
    last_y: f32,
    events_loop: glutin::EventsLoop,
    gl_window: glutin::GlWindow,

    shader: Shader,
    loc_projection: i32,
    loc_view: i32,

    scene: Scene,

    delta_time: f64, // seconds
    last_frame: Instant,
}

impl GltfViewer {
    pub fn new(source: &str) -> GltfViewer {
        let camera = Camera {
            // TODO!: position.z - bounding box length
            position: Point3::new(0.0, 0.0, 2.0),
            zoom: 60.0,
            ..Camera::default()
        };

        let first_mouse = true;
        let last_x: f32 = SCR_WIDTH as f32 / 2.0;
        let last_y: f32 = SCR_HEIGHT as f32 / 2.0;

        // glutin: initialize and configure
        let events_loop = glutin::EventsLoop::new();

        // TODO?: hints for 4.1, core profile, forward compat
        let window = glutin::WindowBuilder::new()
                .with_title("gltf-viewer")
                // TODO: configurable initial dimensions
                .with_dimensions(SCR_WIDTH, SCR_HEIGHT);

        let context = glutin::ContextBuilder::new()
            .with_vsync(true);
        let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

        unsafe {
            gl_window.make_current().unwrap();
        }

        // TODO!: capturing - on click or uncapture somehow?
        // TODO!: find solution for macOS - see https://github.com/tomaka/glutin/issues/226
         #[cfg(target_os = "macos")]
        let _ = gl_window.set_cursor_state(CursorState::Hide);
         #[cfg(not(target_os = "macos"))]
        let _ = gl_window.set_cursor_state(CursorState::Grab);


        // gl: load all OpenGL function pointers
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

        let (shader, loc_projection, loc_view) = unsafe {
            gl::ClearColor(0.1, 1.0, 0.3, 1.0);
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
            camera,
            first_mouse, last_x, last_y,

            events_loop,
            gl_window,

            shader, loc_projection, loc_view,

            scene: Self::load(source),

            delta_time: 0.0, // seconds
            last_frame: Instant::now(),
        }
    }

    pub fn load(source: &str) -> Scene {
        let mut start_time = Instant::now();
        let gltf =
            if source.starts_with("http") {
                let http_source = HttpSource::new(source);
                let import = gltf::Import::custom(http_source, Default::default());
                let gltf = import_gltf(import);
                println!(); // to end the "progress dots"
                gltf
            }
            else {
                let import = gltf::Import::from_path(source);
                import_gltf(import)
            };

        print_elapsed("Imported glTF in ", &start_time);
        start_time = Instant::now();

        // load first scene
        let scene = Scene::from_gltf(gltf.scenes().nth(0).unwrap());
        print_elapsed("Loaded scene in", &start_time);
        println!("Nodes: {:<2}\nMeshes: {:<2}",
            gltf.nodes().count(),
            scene.meshes.len());

        scene
    }

    pub fn start_render_loop(&mut self, screenshot: bool) {
        let mut render_timer = FrameTimer::new("rendering", 300);
        loop {
            // per-frame time logic
            // NOTE: Deliberately ignoring the seconds of `elapsed()`
            self.delta_time = (self.last_frame.elapsed().subsec_nanos() as f64) / 1_000_000_000.0;
            self.last_frame = Instant::now();

            let keep_running = process_events(
                &mut self.events_loop, &self.gl_window,
                &mut self.first_mouse, &mut self.last_x, &mut self.last_y,
                &mut self.camera);
            if !keep_running { break }
            self.camera.update(self.delta_time); // navigation

            // render
            unsafe {
                render_timer.start();

                gl::ClearColor(0.1, 0.2, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                self.shader.use_program();

                // view/projection transformations
                // TODO!: only re-compute/set perspective on Zoom changes (also view?)
                let projection: Matrix4<f32> = perspective(Deg(self.camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.01, 1000.0);
                let view = self.camera.get_view_matrix();
                self.shader.set_mat4(self.loc_projection, &projection);
                self.shader.set_mat4(self.loc_view, &view);

                self.scene.draw(&mut self.shader);

                render_timer.end();
            }

            self.gl_window.swap_buffers().unwrap();

            // TODO!: implement real screenshotting
            if screenshot {
                // HACK: render for a sec for test script that loads all samples
                if render_timer.frame_times.len() > 60 {
                    return
                }
            }
        }
    }
}

fn import_gltf<S: gltf::import::Source>(import: gltf::Import<S>) -> gltf::Gltf {
    match import.sync() {
        Ok(gltf) => gltf,
        Err(err) => {
            println!("glTF import failed: {:?}", err);
            std::process::exit(1);
        }
    }
}

fn process_events(
    events_loop: &mut glutin::EventsLoop,
    gl_window: &glutin::GlWindow,
    first_mouse: &mut bool,
    last_x: &mut f32,
    last_y: &mut f32,
    mut camera: &mut Camera) -> bool
{
    let mut keep_running = true;
    events_loop.poll_events(|event| {
        match event {
            glutin::Event::WindowEvent{ event, .. } => match event {
                WindowEvent::Closed => keep_running = false,
                WindowEvent::Resized(w, h) => gl_window.resize(w, h), // TODO!: handle aspect ratio changes
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
