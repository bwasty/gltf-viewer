extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate gltf;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;

use cgmath::{Deg, Matrix4, Point3, Vector3};
use gfx::{texture, Factory};
use gfx::traits::Device;
use gfx::traits::FactoryExt;

mod pipeline;
use pipeline::*;

mod renderer;
use renderer::*;

mod cube;
mod gltf_loader;
use gltf_loader::*;

const CLEAR_COLOR: [f32; 4] = [0., 0., 0., 1.0];

// TODO!: TMP
// #[allow(unreachable_code)]
pub fn main() {
    load_file("src/data/Box.gltf");
    // load_file("src/data/minimal.gltf");
    // load_file("../gltf/glTF-Sample-Models/2.0/BoomBox/glTF/BoomBox.gltf");
    // return;
    

    ///////////////
    let events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("glTF Viewer".to_string())
        .with_dimensions(1024, 768)
        .with_vsync();
    let (window, mut device, mut factory, main_color_target, main_depth_target) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);

    let (cur_width, cur_height) = window.get_inner_size_points().unwrap();
    let aspect_ratio = cur_width as f32 / cur_height as f32;

    ///
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let vertex_data = cube::vertex_data();
    let index_data: &[u16] = &cube::index_data();

    let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, index_data);

    let texels = [[0x20, 0xA0, 0xC0, 0x00]];
    let (_, texture_view) = factory.create_texture_immutable::<gfx::format::Rgba8>(
        texture::Kind::D2(1, 1, texture::AaMode::Single), &[&texels]
        ).unwrap();

    let sinfo = texture::SamplerInfo::new(
        texture::FilterMethod::Bilinear,
        texture::WrapMode::Clamp);

    // TODO!: other shader variants
    let pso = factory.create_pipeline_simple(
        include_bytes!("shader/cube_150.glslv"),
        include_bytes!("shader/cube_150.glslf"),
        cube::pipe::new()
    ).unwrap();

    let proj = cgmath::perspective(Deg(45.0f32), aspect_ratio, 1.0, 10.0);

    let mut data = cube::pipe::Data {
        vbuf: vbuf,
        transform: (proj * default_view()).into(),
        locals: factory.create_constant_buffer(1),
        color: (texture_view, factory.create_sampler(sinfo)),
        out_color: main_color_target,
        out_depth: main_depth_target,
    };

    let mut running = true;
    while running {
        // fetch events
        events_loop.poll_events(|glutin::Event::WindowEvent{ event, ..}| {
            match event {
                glutin::WindowEvent::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape), _) |
                glutin::WindowEvent::Closed => running = false,
                glutin::WindowEvent::Resized(_width, _height) => {
                    gfx_window_glutin::update_views(&window, &mut data.out_color, &mut data.out_depth);
                },
                _ => {},
            }
        });

        let locals = cube::Locals { transform: data.transform };
        encoder.update_constant_buffer(&data.locals, &locals);
        encoder.clear(&data.out_color, CLEAR_COLOR);
        encoder.clear_depth(&data.out_depth, 1.0);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}


