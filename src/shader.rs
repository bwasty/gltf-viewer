use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
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
    pub fn new(vertex_path: &str, fragment_path: &str, defines: &[String]) -> Shader {
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

        Self::from_source(&vertex_code, &fragment_code, defines)
    }

    pub fn from_source(vertex_code: &str, fragment_code: &str, defines: &[String]) -> Shader {
        let mut shader = Shader {
            id: 0,
            uniform_location_cache: HashMap::new()
        };

        let vertex_code = Self::add_defines(vertex_code, defines);
        let v_shader_code = CString::new(vertex_code.as_bytes()).unwrap();
        let fragment_code = Self::add_defines(fragment_code, defines);
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

    fn add_defines(source: &str, defines: &[String]) -> String {
        // insert preprocessor defines after #version if exists
        // (#version must occur before any other statement in the program)
        let defines = defines.iter()
            .map(|define| format!("#define {}", define))
            .collect::<Vec<_>>()
            .join("\n");
        let mut lines: Vec<_> = source.lines().collect();
        if let Some(version_line) = lines.iter().position(|l| l.starts_with("#version")) {
            lines.insert(version_line+1, &defines);
        }
        else {
            lines.insert(0, &defines);
        }
        lines.join("\n")
    }

    /// activate the shader
    /// ------------------------------------------------------------------------
    pub unsafe fn use_program(&self) {
        gl::UseProgram(self.id)
    }

    /// utility uniform functions
    /// ------------------------------------------------------------------------
    pub unsafe fn set_bool(&self, location: i32, value: bool) {
        gl::Uniform1i(location, value as i32);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_int(&self, location: i32, value: i32) {
        gl::Uniform1i(location, value);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_float(&self, location: i32, value: f32) {
        gl::Uniform1f(location, value);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vector3(&self, location: i32, value: &Vector3<f32>) {
        gl::Uniform3fv(location, 1, value.as_ptr());
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vector4(&self, location: i32, value: &Vector4<f32>) {
        gl::Uniform4fv(location, 1, value.as_ptr());
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_vec3(&self, location: i32, x: f32, y: f32, z: f32) {
        gl::Uniform3f(location, x, y, z);
    }
    /// ------------------------------------------------------------------------
    pub unsafe fn set_mat4(&self, location: i32, mat: &Matrix4<f32>) {
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
            info!("uniform '{}' unknown for shader {}", name, self.id);
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

bitflags! {
    /// Flags matching the defines in the PBR shader
    pub struct ShaderFlags: u16 {
        // vertex shader + fragment shader
        const HAS_NORMALS           = 1;
        const HAS_TANGENTS          = 1 << 1;
        const HAS_UV                = 1 << 2;
        // TODO!: the shader doesn't have colors yet
        const HAS_COLORS            = 1 << 3;

        // fragment shader only
        const USE_IBL               = 1 << 4;
        const HAS_BASECOLORMAP      = 1 << 5;
        const HAS_NORMALMAP         = 1 << 6;
        const HAS_EMISSIVEMAP       = 1 << 7;
        const HAS_METALROUGHNESSMAP = 1 << 8;
        const HAS_OCCLUSIONMAP      = 1 << 9;
        const USE_TEX_LOD           = 1 << 10;
    }
}

impl ShaderFlags {
    pub fn as_strings(&self) -> Vec<String> {
        (0..15)
            .map(|i| 1u16 << i)
            .filter(|i| self.bits & i != 0)
            .map(|i| format!("{:?}", ShaderFlags::from_bits_truncate(i)))
            .collect()
    }
}

#[allow(non_snake_case)]
pub struct PbrUniformLocations {
    // uniform locations
    // TODO!: UBO for matrices, camera, light(s)?
    pub u_MVPMatrix: i32,
    pub u_ModelMatrix: i32,
    pub u_Camera: i32,

    pub u_LightDirection: i32,
    pub u_LightColor: i32,

    pub u_DiffuseEnvSampler: i32,
    pub u_SpecularEnvSampler: i32,
    pub u_brdfLUT: i32,

    pub u_BaseColorSampler: i32,
    pub u_BaseColorFactor: i32,

    pub u_NormalSampler: i32,
    pub u_NormalScale: i32,

    pub u_EmissiveSampler: i32,
    pub u_EmissiveFactor: i32,

    pub u_MetallicRoughnessSampler: i32,
    pub u_MetallicRoughnessValues: i32,

    pub u_OcclusionSampler: i32,
    pub u_OcclusionStrength: i32,

    // debugging flags used for shader output of intermediate PBR variables
    pub u_ScaleDiffBaseMR: i32,
    pub u_ScaleFGDSpec: i32,
    pub u_ScaleIBLAmbient: i32,
}

pub struct PbrShader {
    pub shader: Shader,
    pub flags: ShaderFlags,
    pub uniforms: PbrUniformLocations,
}

impl PbrShader {
    pub fn new(flags: ShaderFlags) -> Self {
        // TODO!!!: switch before release! + find better way...override?
        // let mut shader = Shader::from_source(
        //     include_str!("shaders/pbr-vert.glsl"),
        //     include_str!("shaders/pbr-frag.glsl")
        //     &flags.as_strings());

        // NOTE: shader debug version
        let mut shader = Shader::new(
            "src/shaders/pbr-vert.glsl",
            "src/shaders/pbr-frag.glsl",
            &flags.as_strings());

        let uniforms = unsafe {
            PbrUniformLocations {
                u_MVPMatrix: shader.uniform_location("u_MVPMatrix"),
                u_ModelMatrix: shader.uniform_location("u_ModelMatrix"),
                u_Camera: shader.uniform_location("u_Camera"),

                u_LightDirection: shader.uniform_location("u_LightDirection"),
                u_LightColor: shader.uniform_location("u_LightColor"),

                u_DiffuseEnvSampler: shader.uniform_location("u_DiffuseEnvSampler"),
                u_SpecularEnvSampler: shader.uniform_location("u_SpecularEnvSampler"),
                u_brdfLUT: shader.uniform_location("u_brdfLUT"),

                u_BaseColorSampler: shader.uniform_location("u_BaseColorSampler"),
                u_BaseColorFactor: shader.uniform_location("u_BaseColorFactor"),

                u_NormalSampler: shader.uniform_location("u_NormalSampler"),
                u_NormalScale: shader.uniform_location("u_NormalScale"),

                u_EmissiveSampler: shader.uniform_location("u_EmissiveSampler"),
                u_EmissiveFactor: shader.uniform_location("u_EmissiveFactor"),

                u_MetallicRoughnessSampler: shader.uniform_location("u_MetallicRoughnessSampler"),
                u_MetallicRoughnessValues: shader.uniform_location("u_MetallicRoughnessValues"),

                u_OcclusionSampler: shader.uniform_location("u_OcclusionSampler"),
                u_OcclusionStrength: shader.uniform_location("u_OcclusionStrength"),

                u_ScaleDiffBaseMR: shader.uniform_location("u_ScaleDiffBaseMR"),
                u_ScaleFGDSpec: shader.uniform_location("u_ScaleFGDSpec"),
                u_ScaleIBLAmbient: shader.uniform_location("u_ScaleIBLAmbient"),
            }
        };

        Self {
            shader,
            flags,
            uniforms
        }
    }
}
