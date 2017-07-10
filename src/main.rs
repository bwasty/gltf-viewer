#![allow(dead_code)]
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

mod shader;
use shader::Shader;
mod camera;
use camera::Camera;
use camera::CameraMovement::*;
mod macros;
mod mesh;

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
        .get_matches();
    let source = args.value_of("FILE/URL").unwrap();

    let mut camera = Camera {
        // TODO: position.z - bounding box length
        position: Point3::new(0.0, 0.0, 2.0),
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
    // TODO!: capture on click or sth?
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1)); // V-sync

    // gl: load all OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader, scene) = unsafe {
        gl::Enable(gl::DEPTH_TEST);

        let shader = Shader::new("src/shaders/simple.vs", "src/shaders/simple.fs");

        let gltf = match gltf::Import::from_path(source).sync() {
            Ok(gltf) => gltf,
            Err(err) => {
                println!("Error: {:?}", err);
                panic!();
            }
        };
        // load first scene
        let scene = Scene::from_gltf(gltf.scenes().nth(0).unwrap());

        // draw in wireframe
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        (shader, scene)
    };

    // render loop
    while !window.should_close() {
        // per-frame time logic
        let current_frame = glfw.get_time() as f32;
        delta_time = current_frame - last_frame;
        last_frame = current_frame;

        process_events(&events, &mut first_mouse, &mut last_x, &mut last_y, &mut camera);
        process_input(&mut window, delta_time, &mut camera);

        // render
        unsafe {
            gl::ClearColor(0.1, 0.2, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            shader.use_program();

            // view/projection transformations
            let projection: Matrix4<f32> = perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.01, 1000.0);
            let view = camera.get_view_matrix();
            shader.set_mat4(c_str!("projection"), &projection);
            shader.set_mat4(c_str!("view"), &view);

            scene.draw(&shader);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        window.swap_buffers();
        glfw.poll_events();
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


