use gfx_device_gl::Factory;

pub trait Renderer {
    fn new(factory: Factory) -> Self;
    fn render();
}
