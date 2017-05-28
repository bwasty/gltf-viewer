use pipeline::*;
use gfx;
use gfx::{texture, Factory, Device};
use gfx_device_gl;
use gfx::traits::FactoryExt;
use cgmath::{Deg, perspective};
use glutin;

use renderer::{ Renderer, default_view };

const CLEAR_COLOR: [f32; 4] = [0., 0., 0., 1.0];

pub fn vertex_data() -> [CubeVertex; 24] {
    [
        // top (0, 0, 1)
        CubeVertex::new([-1, -1,  1], [0, 0]),
        CubeVertex::new([ 1, -1,  1], [1, 0]),
        CubeVertex::new([ 1,  1,  1], [1, 1]),
        CubeVertex::new([-1,  1,  1], [0, 1]),
        // bottom (0, 0, -1)
        CubeVertex::new([-1,  1, -1], [1, 0]),
        CubeVertex::new([ 1,  1, -1], [0, 0]),
        CubeVertex::new([ 1, -1, -1], [0, 1]),
        CubeVertex::new([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        CubeVertex::new([ 1, -1, -1], [0, 0]),
        CubeVertex::new([ 1,  1, -1], [1, 0]),
        CubeVertex::new([ 1,  1,  1], [1, 1]),
        CubeVertex::new([ 1, -1,  1], [0, 1]),
        // left (-1, 0, 0)
        CubeVertex::new([-1, -1,  1], [1, 0]),
        CubeVertex::new([-1,  1,  1], [0, 0]),
        CubeVertex::new([-1,  1, -1], [0, 1]),
        CubeVertex::new([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        CubeVertex::new([ 1,  1, -1], [1, 0]),
        CubeVertex::new([-1,  1, -1], [0, 0]),
        CubeVertex::new([-1,  1,  1], [0, 1]),
        CubeVertex::new([ 1,  1,  1], [1, 1]),
        // back (0, -1, 0)
        CubeVertex::new([ 1, -1,  1], [0, 0]),
        CubeVertex::new([-1, -1,  1], [1, 0]),
        CubeVertex::new([-1, -1, -1], [1, 1]),
        CubeVertex::new([ 1, -1, -1], [0, 1]),
    ]
}

pub fn index_data() -> [u16; 36] {
    [
         0,  1,  2,  2,  3,  0, // top
         4,  5,  6,  6,  7,  4, // bottom
         8,  9, 10, 10, 11,  8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ]
}

// Declare the vertex format suitable for drawing,
// as well as the constants used by the shaders
// and the pipeline state object format.
// Notice the use of FixedPoint.
gfx_defines!{
    vertex CubeVertex {
        pos: [f32; 4] = "a_Pos",
        tex_coord: [f32; 2] = "a_TexCoord",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_Transform",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<CubeVertex> = (),
        transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        color: gfx::TextureSampler<[f32; 4]> = "t_Color",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}


impl CubeVertex {
    pub fn new(p: [i8; 3], t: [i8; 2]) -> CubeVertex {
        CubeVertex {
            pos: [p[0] as f32, p[1] as f32, p[2] as f32, 1.0],
            tex_coord: [t[0] as f32, t[1] as f32],
        }
    }
}

struct CubeRenderer {
    factory: gfx_device_gl::Factory,
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    pso: gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
    data: pipe::Data<gfx_device_gl::Resources>,
}

impl Renderer for CubeRenderer {
    fn new(mut factory: gfx_device_gl::Factory, aspect_ratio: f32, 
        color_target: gfx::handle::RenderTargetView<gfx_device_gl::Resources, (gfx::format::R8_G8_B8_A8, gfx::format::Unorm)>, 
        depth_target: gfx::handle::DepthStencilView<gfx_device_gl::Resources, (gfx::format::D24_S8, gfx::format::Unorm)>) -> Self 
    {
        let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        let vertex_data = vertex_data();
        let index_data: &[u16] = &index_data();
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
            pipe::new()
        ).unwrap();

        let proj = perspective(Deg(45.0f32), aspect_ratio, 1.0, 10.0);

        let mut data = pipe::Data {
            vbuf: vbuf,
            transform: (proj * default_view()).into(),
            locals: factory.create_constant_buffer(1),
            color: (texture_view, factory.create_sampler(sinfo)),
            out_color: color_target,
            out_depth: depth_target,
        };

        let this = CubeRenderer { factory, encoder, pso, slice, data };
        this
    }

    fn render<D, R, C>(&mut self, device: &mut D, window: &glutin::Window)
        where D: gfx::Device 
    {
        let locals = Locals { transform: self.data.transform };
        self.encoder.update_constant_buffer(&self.data.locals, &locals);
        self.encoder.clear(&self.data.out_color, CLEAR_COLOR);
        self.encoder.clear_depth(&self.data.out_depth, 1.0);
        self.encoder.draw(&self.slice, &self.pso, &self.data);
        self.encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
