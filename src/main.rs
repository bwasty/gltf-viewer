#![allow(dead_code)]
// #![allow(unused_features)]
// #![feature(test)]
#[macro_use] extern crate clap;
extern crate cgmath;
use cgmath::{Matrix4, Point3, Deg, perspective};
// use cgmath::prelude::*;

extern crate gl;
extern crate glfw;
use self::glfw::{Context, Key, Action};
extern crate gltf;
extern crate image;

extern crate futures;

use clap::{Arg, App};

use std::sync::mpsc::Receiver;
use std::ffi::CStr;
use std::time::SystemTime;

mod shader;
use shader::Shader;
mod camera;
use camera::Camera;
use camera::CameraMovement::*;
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

    let mut camera = Camera {
        // TODO!: position.z - bounding box length
        position: Point3::new(0.0, 0.0, 1.0),
        zoom: 60.0,
        ..Camera::default()
    };

    let mut first_mouse = true;
    let mut last_x: f32 = SCR_WIDTH as f32 / 2.0;
    let mut last_y: f32 = SCR_HEIGHT as f32 / 2.0;

    // timing
    let mut delta_time: f32;
    let mut last_frame: f32 = 0.0;

    // glfw: initialize and configure
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 1)); // max on macOS
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // glfw window creation
    // TODO: configurable initial dimensions
    let (mut window, events) = glfw.create_window(SCR_WIDTH, SCR_HEIGHT, "gltf-viewer", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    // TODO: capture on click or sth?
    // window.set_cursor_mode(glfw::CursorMode::Disabled);

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1)); // V-sync
    // glfw.set_swap_interval(glfw::SwapInterval::None);

    // gl: load all OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader, scene) = unsafe {
        gl::Enable(gl::DEPTH_TEST);

        let shader = Shader::from_source(
            include_str!("shaders/simple.vs"),
            include_str!("shaders/simple.fs"));

        // NOTE: shader debug version
        // let shader = Shader::new(
        //     "src/shaders/simple.vs",
        //     "src/shaders/simple.fs");

        let mut start_time = SystemTime::now();
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
        start_time = SystemTime::now();

        // load first scene
        let scene = Scene::from_gltf(gltf.scenes().nth(0).unwrap());
        print_elapsed("Loaded scene in", &start_time);
        println!("Nodes: {:<2}\nMeshes: {:<2}",
            gltf.nodes().count(),
            scene.meshes.len());

        // draw in wireframe
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        (shader, scene)
    };

    let mut frame_timer = FrameTimer::new("frame", 300);
    let mut render_timer = FrameTimer::new("rendering", 300);

    // render loop
    while !window.should_close() {
        frame_timer.start();

        // per-frame time logic
        let current_frame = glfw.get_time() as f32;
        delta_time = current_frame - last_frame;
        last_frame = current_frame;

        // poll events - perf note: this is slow while navigating! up to 1ms avg, 5ms max
        glfw.poll_events();

        process_events(&events, &mut first_mouse, &mut last_x, &mut last_y, &mut camera);
        process_input(&mut window, delta_time, &mut camera);

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
            shader.set_mat4(c_str!("projection"), &projection);
            shader.set_mat4(c_str!("view"), &view);

            scene.draw(&shader);

            render_timer.end();
        }

        frame_timer.end();

        window.swap_buffers();

        // TODO!: implement screenshotting
        if screenshot { return }
    }

}

fn process_events(events: &Receiver<(f64, glfw::WindowEvent)>,
                  first_mouse: &mut bool,
                  last_x: &mut f32,
                  last_y: &mut f32,
                  camera: &mut Camera) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            glfw::WindowEvent::CursorPos(xpos, ypos) => {
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
            }
            glfw::WindowEvent::Scroll(_xoffset, yoffset) => {
                camera.process_mouse_scroll(yoffset as f32);
            }
            _ => {}
        }
    }
}

fn process_input(window: &mut glfw::Window, delta_time: f32, camera: &mut Camera) {
    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true)
    }

    if window.get_key(Key::W) == Action::Press {
        camera.process_keyboard(FORWARD, delta_time);
    }
    if window.get_key(Key::S) == Action::Press {
        camera.process_keyboard(BACKWARD, delta_time);
    }
    if window.get_key(Key::A) == Action::Press {
        camera.process_keyboard(LEFT, delta_time);
    }
    if window.get_key(Key::D) == Action::Press {
        camera.process_keyboard(RIGHT, delta_time);
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
