#![allow(dead_code)]
// #![allow(unused_features)]
// #![feature(test)]
#[macro_use] extern crate clap;
extern crate cgmath;
use cgmath::{Matrix4, Point3, Deg, perspective};
// use cgmath::prelude::*;

extern crate gl;

extern crate glutin;
use glutin::GlContext;

extern crate gltf;
extern crate image;

extern crate futures;

use clap::{Arg, App};

// use std::sync::mpsc::Receiver;
use std::time::Instant;

mod shader;
use shader::Shader;
mod camera;
use camera::Camera;
// use camera::CameraMovement::*;
mod macros;
mod http_source;
use http_source::HttpSource;
mod utils;
use utils::{print_elapsed, FrameTimer, gl_check_error};

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

    let camera = Camera {
        // TODO!: position.z - bounding box length
        position: Point3::new(0.0, 0.0, 1.0),
        zoom: 60.0,
        ..Camera::default()
    };

    // TODO!!
    // let mut first_mouse = true;
    // let mut last_x: f32 = SCR_WIDTH as f32 / 2.0;
    // let mut last_y: f32 = SCR_HEIGHT as f32 / 2.0;

    // // timing
    // let mut delta_time: f32;
    // let mut last_frame: f32 = 0.0;

    // glutin: initialize and configure
    let mut events_loop = glutin::EventsLoop::new();
    // TODO!?: hints for 4.1, core, forward compat

    let window = glutin::WindowBuilder::new()
            .with_title("gltf-viewer")
            // TODO: configurable initial dimensions
            .with_dimensions(SCR_WIDTH, SCR_HEIGHT);
    // TODO!!: capturing - on click or uncapture somehow?

    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    unsafe {
        gl_window.make_current().unwrap();
    }

    // gl: load all OpenGL function pointers
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    let (mut shader, mut scene, loc_projection, loc_view) = unsafe {
            gl::ClearColor(0.1, 1.0, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        gl::Enable(gl::DEPTH_TEST);

        let mut shader = Shader::from_source(
            include_str!("shaders/simple.vs"),
            include_str!("shaders/simple.fs"));

        // NOTE: shader debug version
        // let shader = Shader::new(
        //     "src/shaders/simple.vs",
        //     "src/shaders/simple.fs");

        shader.use_program();
        let loc_projection = shader.uniform_location("projection");
        let loc_view = shader.uniform_location("view");

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

        // draw in wireframe
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        (shader, scene, loc_projection, loc_view)
    };

    // let mut frame_timer = FrameTimer::new("frame", 300);
    let mut render_timer = FrameTimer::new("rendering", 300);

    // render loop
    let mut running = true;
    while running {
        // TODO!!: process_events, process_input
        events_loop.poll_events(|event| {
            // println!("{:?}", event);
            match event {
                glutin::Event::WindowEvent{ event, .. } => match event {
                    glutin::WindowEvent::Closed => running = false,
                    glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                    _ => ()
                },
                _ => ()
            }
        });

        // render
        unsafe {
            render_timer.start();

            gl::ClearColor(0.1, 0.2, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            shader.use_program();

            // view/projection transformations
            // TODO!: only re-compute/set perspective on Zoom changes (also view?)
            let projection: Matrix4<f32> = perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.01, 1000.0);
            let view = camera.get_view_matrix();
            shader.set_mat4(loc_projection, &projection);
            shader.set_mat4(loc_view, &view);

            scene.draw(&mut shader);

            render_timer.end();

            gl_check_error!(); // TODO!!: fix closing issue & remove
        }

        // frame_timer.end();

        gl_window.swap_buffers().unwrap();

        // TODO!: implement screenshotting
        if screenshot { return }
    }
}

// fn process_events(events: &Receiver<(f64, glfw::WindowEvent)>,
//                   first_mouse: &mut bool,
//                   last_x: &mut f32,
//                   last_y: &mut f32,
//                   camera: &mut Camera) {
//     for (_, event) in glfw::flush_messages(events) {
//         match event {
//             glfw::WindowEvent::FramebufferSize(width, height) => {
//                 // make sure the viewport matches the new window dimensions; note that width and
//                 // height will be significantly larger than specified on retina displays.
//                 unsafe { gl::Viewport(0, 0, width, height) }
//             }
//             glfw::WindowEvent::CursorPos(xpos, ypos) => {
//                 let (xpos, ypos) = (xpos as f32, ypos as f32);
//                 if *first_mouse {
//                     *last_x = xpos;
//                     *last_y = ypos;
//                     *first_mouse = false;
//                 }

//                 let xoffset = xpos - *last_x;
//                 let yoffset = *last_y - ypos; // reversed since y-coordinates go from bottom to top

//                 *last_x = xpos;
//                 *last_y = ypos;

//                 camera.process_mouse_movement(xoffset, yoffset, true);
//             }
//             glfw::WindowEvent::Scroll(_xoffset, yoffset) => {
//                 camera.process_mouse_scroll(yoffset as f32);
//             }
//             _ => {}
//         }
//     }
// }

// fn process_input(window: &mut glfw::Window, delta_time: f32, camera: &mut Camera) {
//     if window.get_key(Key::Escape) == Action::Press {
//         window.set_should_close(true)
//     }

//     if window.get_key(Key::W) == Action::Press {
//         camera.process_keyboard(FORWARD, delta_time);
//     }
//     if window.get_key(Key::S) == Action::Press {
//         camera.process_keyboard(BACKWARD, delta_time);
//     }
//     if window.get_key(Key::A) == Action::Press {
//         camera.process_keyboard(LEFT, delta_time);
//     }
//     if window.get_key(Key::D) == Action::Press {
//         camera.process_keyboard(RIGHT, delta_time);
//     }

// }

fn import_gltf<S: gltf::import::Source>(import: gltf::Import<S>) -> gltf::Gltf {
    match import.sync() {
        Ok(gltf) => gltf,
        Err(err) => {
            println!("glTF import failed: {:?}", err);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_struct_sizes() {
        // run witn `cargo test -- --nocapture`
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
