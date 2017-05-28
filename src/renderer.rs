use gfx;
use gfx_device_gl::{ Factory, Resources, Device };
use glutin;
use cgmath::{ Matrix4, Point3, Vector3};

pub trait Renderer {
    fn new(factory: Factory, aspect_ratio: f32,color_target: gfx::handle::RenderTargetView<Resources, (gfx::format::R8_G8_B8_A8, gfx::format::Unorm)>, 
        depth_target: gfx::handle::DepthStencilView<Resources, (gfx::format::D24_S8, gfx::format::Unorm)>) -> Self;
    fn render<D, R: gfx::Resources, C: gfx::CommandBuffer<R>>(&mut self, device: &mut D, window: &glutin::Window)
        where D: gfx::Device<Resources=R, CommandBuffer=C>;
}

pub fn default_view() -> Matrix4<f32> {
    Matrix4::look_at(
        Point3::new(1.5f32, -5.0, 3.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    )
}
