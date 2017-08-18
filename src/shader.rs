use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::iter;
use std::ptr;
use std::str;

use gl;
use gl::types::*;

use cgmath::{Matrix, Matrix4, Vector3, Vector4};
use cgmath::prelude::*;

pub struct Shader {
    pub id: u32,
    uniform_location_cache: HashMap<&'static str, i32>
}

impl Shader {
    pub fn new(vertex_path: &str, fragment_path: &str) -> Shader {
        // 1. retrieve the vertex/fragment source code from filesystem
        let mut v_shader_file = File::open(vertex_path).expect(&format!("Failed to open {}", vertex_path));
        let mut f_shader_file = File::open(fragment_path).expect(&format!("Failed to open {}", fragment_path));
        let mut vertex_code = String::new();
        let mut fragment_code = String::new();
        v_shader_file
            .read_to_string(&mut vertex_code)
            .expect("Failed to read vertex shader");
        f_shader_file
            .read_to_string(&mut fragment_code)
            .expect("Failed to read fragment shader");

        Self::from_source(&vertex_code, &fragment_code)
    }

    pub fn from_source(vertex_code: &str, fragment_code: &str) -> Shader {
        let mut shader = Shader { id: 0, uniform_location_cache: HashMap::new() };

        let v_shader_code = CString::new(vertex_code.as_bytes()).unwrap();
        let f_shader_code = CString::new(fragment_code.as_bytes()).unwrap();

        // 2. compile shaders
        unsafe {
            // vertex shader
            let vertex = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex, 1, &v_shader_code.as_ptr(), ptr::null());
            gl::CompileShader(vertex);
            shader.check_compile_errors(vertex, "VERTEX");
            // fragment Shader
            let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment, 1, &f_shader_code.as_ptr(), ptr::null());
            gl::CompileShader(fragment);
            shader.check_compile_errors(fragment, "FRAGMENT");
            // shader Program
            let id = gl::CreateProgram();
            gl::AttachShader(id, vertex);
            gl::AttachShader(id, fragment);
            gl::LinkProgram(id);
            shader.check_compile_errors(id, "PROGRAM");
            // delete the shaders as they're linked into our program now and no longer necessary
            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
            shader.id = id;
        }

        shader
    }

    fn add_defines(source: &str, defines: &[&'static str]) -> String {
        defines.iter()
            .map(|define| format!("#define {}", define))
            .chain(iter::once(source.into()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// activate the shader
    /// ------------------------------------------------------------------------
    pub unsafe fn use_program(&self) {
        gl::UseProgram(self.id)
    }

    // TODO: add variants that take a loc directly?
    /// utility uniform functions
    /// ------------------------------------------------------------------------
    pub unsafe fn set_bool(&mut self, location: i32, value: bool) {
        gl::Uniform1i(location, value as i32);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_int(&mut self, location: i32, value: i32) {
        gl::Uniform1i(location, value);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_float(&mut self, location: i32, value: f32) {
        gl::Uniform1f(location, value);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vector3(&mut self, location: i32, value: &Vector3<f32>) {
        gl::Uniform3fv(location, 1, value.as_ptr());
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vector4(&mut self, location: i32, value: &Vector4<f32>) {
        gl::Uniform4fv(location, 1, value.as_ptr());
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vec3(&mut self, location: i32, x: f32, y: f32, z: f32) {
        gl::Uniform3f(location, x, y, z);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_mat4(&mut self, location: i32, mat: &Matrix4<f32>) {
        gl::UniformMatrix4fv(location, 1, gl::FALSE, mat.as_ptr());
    }

    /// get uniform location with caching
    pub unsafe fn uniform_location(&mut self, name: &'static str) -> i32 {
        if let Some(loc) = self.uniform_location_cache.get(name) {
            return *loc;
        }

        let c_name = CString::new(name).unwrap();
        let loc = gl::GetUniformLocation(self.id, c_name.as_ptr());
        if loc == -1 {
            warn!("uniform '{}' unknown for shader {}", name, self.id);
        }
        self.uniform_location_cache.insert(name, loc);
        loc
    }

    /// utility function for checking shader compilation/linking errors.
    /// ------------------------------------------------------------------------
    unsafe fn check_compile_errors(&self, shader: u32, type_: &str) {
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(1024);
        info_log.set_len(1024 - 1); // subtract 1 to skip the trailing null character
        if type_ != "PROGRAM" {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            let log_type = if success == gl::TRUE as GLint { "WARNING" } else { "ERROR" };
            let mut length = 0;
            gl::GetShaderInfoLog(shader, 1024, &mut length, info_log.as_mut_ptr() as *mut GLchar);
            if length == 0 { return }
            panic!("{}::SHADER_COMPILATION_{} of type: {}\n{}",
                      log_type, log_type,
                      type_,
                      str::from_utf8(&info_log[0..length as usize]).unwrap());

        } else {
            gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
            let log_type = if success == gl::TRUE as GLint { "WARNING" } else { "ERROR" };
            let mut length = 0;
            gl::GetProgramInfoLog(shader, 1024, &mut length, info_log.as_mut_ptr() as *mut GLchar);
            if length == 0 { return }
            panic!("{}::PROGRAM_LINKING_{} of type: {}\n{}",
                      log_type, log_type,
                      type_,
                      str::from_utf8(&info_log[0..length as usize]).unwrap());
        }

    }
}
