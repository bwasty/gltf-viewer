use cgmath::{Matrix, Matrix4, Vector3, Vector4};
use cgmath::prelude::*;

pub trait UniformHelpers {
    unsafe fn use_program(&self);
    unsafe fn set_bool(&self, location: i32, value: bool);
    unsafe fn set_int(&self, location: i32, value: i32);
    unsafe fn set_float(&self, location: i32, value: f32);
    unsafe fn set_vector3(&self, location: i32, value: &Vector3<f32>);
    unsafe fn set_vector4(&self, location: i32, value: &Vector4<f32>);
    unsafe fn set_vec2(&self, location: i32, x: f32, y: f32);
    unsafe fn set_vec3(&self, location: i32, x: f32, y: f32, z: f32);
    unsafe fn set_mat4(&self, location: i32, mat: &Matrix4<f32>);
    unsafe fn uniform_location(&mut self, name: &'static str) -> i32;
}

pub fn read_vertex_code(_vertex_path: &str) -> String {
    String::from(include_str!("../../shaders/pbr-vert.glsl"))
}
pub fn read_fragment_code(_fragment_path: &str) -> String {
    String::from(include_str!("../../shaders/pbr-frag.glsl"))
}

// 2. compile shaders
pub unsafe fn compile_shader_and_get_id(v_shader_code: &str, f_shader_code: &str) -> Result<u32,String> {
    // vertex shader
    // let vertex = gl::CreateShader(gl::VERTEX_SHADER);
    // gl::ShaderSource(vertex, 1, &v_shader_code.as_ptr(), ptr::null());
    // gl::CompileShader(vertex);
    // check_compile_errors(vertex, "VERTEX");
    // // fragment Shader
    // let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
    // gl::ShaderSource(fragment, 1, &f_shader_code.as_ptr(), ptr::null());
    // gl::CompileShader(fragment);
    // check_compile_errors(fragment, "FRAGMENT");
    // // shader Program
    // let id = gl::CreateProgram();
    // gl::AttachShader(id, vertex);
    // gl::AttachShader(id, fragment);
    // gl::LinkProgram(id);
    // check_compile_errors(id, "PROGRAM");
    // // delete the shaders as they're linked into our program now and no longer necessary
    // gl::DeleteShader(vertex);
    // gl::DeleteShader(fragment);
    // Ok(id)
    Ok(0) // TODO
}


impl UniformHelpers for crate::shader::Shader {
    /// activate the shader
    /// ------------------------------------------------------------------------
    unsafe fn use_program(&self) {
        // gl::UseProgram(self.id)
    }

    /// utility uniform functions
    /// ------------------------------------------------------------------------
    #[allow(dead_code)]
    unsafe fn set_bool(&self, location: i32, value: bool) {
        // gl::Uniform1i(location, value as i32);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_int(&self, location: i32, value: i32) {
        // gl::Uniform1i(location, value);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_float(&self, location: i32, value: f32) {
        // gl::Uniform1f(location, value);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vector3(&self, location: i32, value: &Vector3<f32>) {
        // gl::Uniform3fv(location, 1, value.as_ptr());
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vector4(&self, location: i32, value: &Vector4<f32>) {
        // gl::Uniform4fv(location, 1, value.as_ptr());
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vec2(&self, location: i32, x: f32, y: f32) {
        // gl::Uniform2f(location, x, y);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vec3(&self, location: i32, x: f32, y: f32, z: f32) {
        // gl::Uniform3f(location, x, y, z);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_mat4(&self, location: i32, mat: &Matrix4<f32>) {
        // gl::UniformMatrix4fv(location, 1, gl::FALSE, mat.as_ptr());
    }

    /// get uniform location with caching
    unsafe fn uniform_location(&mut self, name: &'static str) -> i32 {
        // if let Some(loc) = self.uniform_location_cache.get(name) {
        //     return *loc;
        // }

        // let c_name = CString::new(name).unwrap();
        // let loc = gl::GetUniformLocation(self.id, c_name.as_ptr());
        // if loc == -1 {
        //     trace!("uniform '{}' unknown for shader {}", name, self.id);
        // }
        // self.uniform_location_cache.insert(name, loc);
        // loc
        0 // TODO
    }
}
