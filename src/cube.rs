use pipeline::*;
use gfx;
use gfx_device_gl::Factory;

use renderer::Renderer;

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
    factory: Factory
}

impl Renderer for CubeRenderer {
    fn new(factory: Factory) -> Self {
        let this = CubeRenderer { factory };
        this
    }

    fn render() {

    }
}