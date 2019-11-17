use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::ptr;
use std::str;
use gl;
use gl::types::*;
use cgmath::{Matrix, Matrix4, Vector3, Vector4};
use cgmath::prelude::*;
use log::{warn, trace};

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

// 1. retrieve the vertex/fragment source code from filesystem
pub fn read_vertex_code(vertex_path: &str) -> String {
    let mut v_shader_file = File::open(vertex_path).unwrap_or_else(|_| panic!("Failed to open {}", vertex_path));
    let mut vertex_code = String::new();
    v_shader_file
        .read_to_string(&mut vertex_code)
        .expect("Failed to read vertex shader");
    vertex_code
}
pub fn read_fragment_code(fragment_path: &str) -> String {
    let mut f_shader_file = File::open(fragment_path).unwrap_or_else(|_| panic!("Failed to open {}", fragment_path));
    let mut fragment_code = String::new();
    f_shader_file
        .read_to_string(&mut fragment_code)
        .expect("Failed to read fragment shader");
    fragment_code
}

// 2. compile shaders
pub unsafe fn compile_shader_and_get_id(vertex_code: &str, fragment_code: &str) -> Result<u32,String> {
    let v_shader_code = CString::new(vertex_code.as_bytes()).unwrap();
    let f_shader_code = CString::new(fragment_code.as_bytes()).unwrap();
    
    // vertex shader
    let vertex = gl::CreateShader(gl::VERTEX_SHADER);
    gl::ShaderSource(vertex, 1, &v_shader_code.as_ptr(), ptr::null());
    gl::CompileShader(vertex);
    check_compile_errors(vertex, "VERTEX");
    // fragment Shader
    let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
    gl::ShaderSource(fragment, 1, &f_shader_code.as_ptr(), ptr::null());
    gl::CompileShader(fragment);
    check_compile_errors(fragment, "FRAGMENT");
    // shader Program
    let id = gl::CreateProgram();
    gl::AttachShader(id, vertex);
    gl::AttachShader(id, fragment);
    gl::LinkProgram(id);
    check_compile_errors(id, "PROGRAM");
    // delete the shaders as they're linked into our program now and no longer necessary
    gl::DeleteShader(vertex);
    gl::DeleteShader(fragment);
    Ok(id)
}


/// utility function for checking shader compilation/linking errors.
/// ------------------------------------------------------------------------
pub unsafe fn check_compile_errors(shader: u32, type_: &str) {
    let mut success = i32::from(gl::FALSE);
    let mut info_log = Vec::with_capacity(1024);
    info_log.set_len(1024 - 1); // subtract 1 to skip the trailing null character
    if type_ != "PROGRAM" {
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        let log_type = if success == i32::from(gl::TRUE) { "WARNING" } else { "ERROR" };
        let mut length = 0;
        gl::GetShaderInfoLog(shader, 1024, &mut length, info_log.as_mut_ptr() as *mut GLchar);
        if length == 0 { return }
        panic!("{}::SHADER_COMPILATION_{} of type: {}\n{}",
                    log_type, log_type,
                    type_,
                    str::from_utf8(&info_log[0..length as usize]).unwrap());

    } else {
        gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
        let log_type = if success == i32::from(gl::TRUE) { "WARNING" } else { "ERROR" };
        let mut length = 0;
        gl::GetProgramInfoLog(shader, 1024, &mut length, info_log.as_mut_ptr() as *mut GLchar);
        if length == 0 { return }
        warn!("{}::PROGRAM_LINKING_{} of type: {}\n{}",
                    log_type, log_type,
                    type_,
                    str::from_utf8(&info_log[0..length as usize]).unwrap());
    }

}

impl UniformHelpers for crate::shader::Shader {
    /// activate the shader
    /// ------------------------------------------------------------------------
    unsafe fn use_program(&self) {
        gl::UseProgram(self.id)
    }

    /// utility uniform functions
    /// ------------------------------------------------------------------------
    #[allow(dead_code)]
    unsafe fn set_bool(&self, location: i32, value: bool) {
        gl::Uniform1i(location, value as i32);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_int(&self, location: i32, value: i32) {
        gl::Uniform1i(location, value);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_float(&self, location: i32, value: f32) {
        gl::Uniform1f(location, value);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vector3(&self, location: i32, value: &Vector3<f32>) {
        gl::Uniform3fv(location, 1, value.as_ptr());
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vector4(&self, location: i32, value: &Vector4<f32>) {
        gl::Uniform4fv(location, 1, value.as_ptr());
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vec2(&self, location: i32, x: f32, y: f32) {
        gl::Uniform2f(location, x, y);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vec3(&self, location: i32, x: f32, y: f32, z: f32) {
        gl::Uniform3f(location, x, y, z);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_mat4(&self, location: i32, mat: &Matrix4<f32>) {
        gl::UniformMatrix4fv(location, 1, gl::FALSE, mat.as_ptr());
    }

    /// get uniform location with caching
    unsafe fn uniform_location(&mut self, name: &'static str) -> i32 {
        if let Some(loc) = self.uniform_location_cache.get(name) {
            return *loc;
        }

        let c_name = CString::new(name).unwrap();
        let loc = gl::GetUniformLocation(self.id, c_name.as_ptr());
        if loc == -1 {
            trace!("uniform '{}' unknown for shader {}", name, self.id);
        }
        self.uniform_location_cache.insert(name, loc);
        loc
    }
}
