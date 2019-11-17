use crate::render::math::*;
use crate::render::{Primitive,Vertex};


use crate::platform::{UniformHelpers};

pub trait PrimitiveHelpers {
    unsafe fn draw(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3);
    unsafe fn configure_shader(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3);
    unsafe fn setup_primitive(&mut self, vertices: &[Vertex], indices: Option<Vec<u32>>);
}

impl PrimitiveHelpers for Primitive {
    unsafe fn draw(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3) {
        
    }
    unsafe fn configure_shader(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3) {
        
    }
    unsafe fn setup_primitive(&mut self, vertices: &[Vertex], indices: Option<Vec<u32>>) {
        
    }
}
