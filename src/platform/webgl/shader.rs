use std::collections::HashMap;
use std::rc::Rc;

use cgmath::{Matrix, Matrix4, Vector3, Vector4};
use cgmath::prelude::*;
use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram,WebGlUniformLocation};
use web_sys::WebGl2RenderingContext as GL;

use crate::{debug,trace};
use crate::platform::{GltfViewerRenderer};

pub struct ShaderInfo {
    pub program_id: Rc<WebGlProgram>,
    pub uniform_location_cache: Vec<Rc<WebGlUniformLocation>>,
    pub uniform_location_map: HashMap<&'static str, i32>,
}

pub trait UniformHelpers {
    unsafe fn use_program(&self, renderer: &GltfViewerRenderer);
    unsafe fn set_bool(&self, renderer: &GltfViewerRenderer, loc_id: i32, value: bool);
    unsafe fn set_int(&self, renderer: &GltfViewerRenderer, loc_id: i32, value: i32);
    unsafe fn set_float(&self, renderer: &GltfViewerRenderer, loc_id: i32, value: f32);
    unsafe fn set_vector3(&self, renderer: &GltfViewerRenderer, loc_id: i32, value: &Vector3<f32>);
    unsafe fn set_vector4(&self, renderer: &GltfViewerRenderer, loc_id: i32, value: &Vector4<f32>);
    unsafe fn set_vec2(&self, renderer: &GltfViewerRenderer, loc_id: i32, x: f32, y: f32);
    unsafe fn set_vec3(&self, renderer: &GltfViewerRenderer, loc_id: i32, x: f32, y: f32, z: f32);
    unsafe fn set_mat4(&self, renderer: &GltfViewerRenderer, loc_id: i32, mat: &Matrix4<f32>);
    unsafe fn attrib_f32_array(&self, renderer: &GltfViewerRenderer, tag: &str, vertex_data: &[f32], size: i32);
    unsafe fn uniform_location(&mut self, renderer: &mut GltfViewerRenderer, name: &'static str) -> i32;
}

pub fn read_vertex_code(_vertex_path: &str) -> String {
    String::from(include_str!("../../shaders/pbr-vert.glsl"))
}
pub fn read_fragment_code(_fragment_path: &str) -> String {
    String::from(include_str!("../../shaders/pbr-frag.glsl"))
}

// 2. compile shaders
pub unsafe fn compile_shader_and_get_id(v_shader_code: &str, f_shader_code: &str, renderer: &mut GltfViewerRenderer) -> Result<u32,String> {
    let gl = renderer.gl.as_ref();
    
    // vertex shader
    let vertex_shader = gl
        .create_shader(GL::VERTEX_SHADER)
        .ok_or_else(|| "Could not create shader".to_string()).unwrap();
    gl.shader_source(&vertex_shader, v_shader_code);
    gl.compile_shader(&vertex_shader);

    // check errors
    if !gl
        .get_shader_parameter(&vertex_shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        debug!("{}", v_shader_code);
        panic!("vertex shader failed to compile")
    }
    
    // fragment Shader
    let fragment_shader = gl
        .create_shader(GL::FRAGMENT_SHADER)
        .ok_or_else(|| "Could not create shader".to_string()).unwrap();
    gl.shader_source(&fragment_shader, f_shader_code);
    gl.compile_shader(&fragment_shader);

    // check errors
    if !gl
        .get_shader_parameter(&fragment_shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        panic!("fragment shader failed to compile")
    }

    // shader Program
    let program = gl
        .create_program()
        .ok_or_else(|| "Unable to create shader program".to_string())?;

    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);

    gl.link_program(&program);

    // check errors
    if !gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        return Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program".to_string()));
    }

    // TODO delete shader?

    // store and return reference to shader program
    let program_id = renderer.shaders.len() as u32;
    renderer.shaders.push(ShaderInfo {
        program_id: Rc::new(program),
        uniform_location_cache: Vec::new(),
        uniform_location_map: HashMap::new(),
    });

    Ok(program_id)
}


impl UniformHelpers for crate::shader::Shader {
    /// activate the shader
    /// ------------------------------------------------------------------------
    unsafe fn use_program(&self, renderer: &GltfViewerRenderer) {
        renderer.gl.use_program(Some(renderer.shaders[self.id as usize].program_id.as_ref()))
    }

    /// utility uniform functions
    /// ------------------------------------------------------------------------
    #[allow(dead_code)]
    unsafe fn set_bool(&self, renderer: &GltfViewerRenderer, loc_id: i32, value: bool) {
        if loc_id == -1 { return; }
        let location = renderer.shaders[self.id as usize].uniform_location_cache[loc_id as usize].as_ref();
        renderer.gl.uniform1i(Some(location), value as i32);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_int(&self, renderer: &GltfViewerRenderer, loc_id: i32, value: i32) {
        if loc_id == -1 { return; }
        let location = renderer.shaders[self.id as usize].uniform_location_cache[loc_id as usize].as_ref();
        renderer.gl.uniform1i(Some(location), value);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_float(&self, renderer: &GltfViewerRenderer, loc_id: i32, value: f32) {
        if loc_id == -1 { return; }
        let location = renderer.shaders[self.id as usize].uniform_location_cache[loc_id as usize].as_ref();
        renderer.gl.uniform1f(Some(location), value);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vector3(&self, renderer: &GltfViewerRenderer, loc_id: i32, v: &Vector3<f32>) {
        if loc_id == -1 { return; }
        let location = renderer.shaders[self.id as usize].uniform_location_cache[loc_id as usize].as_ref();
        renderer.gl.uniform3fv_with_f32_array(Some(location), &[v.x,v.y,v.z]);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vector4(&self, renderer: &GltfViewerRenderer, loc_id: i32, v: &Vector4<f32>) {
        if loc_id == -1 { return; }
        let location = renderer.shaders[self.id as usize].uniform_location_cache[loc_id as usize].as_ref();
        renderer.gl.uniform4fv_with_f32_array(Some(location), &[v[0],v[1],v[2],v[3]]);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vec2(&self, renderer: &GltfViewerRenderer, loc_id: i32, x: f32, y: f32) {
        if loc_id == -1 { return; }
        let location = renderer.shaders[self.id as usize].uniform_location_cache[loc_id as usize].as_ref();
        renderer.gl.uniform2f(Some(location), x, y);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_vec3(&self, renderer: &GltfViewerRenderer, loc_id: i32, x: f32, y: f32, z: f32) {
        if loc_id == -1 { return; }
        let location = renderer.shaders[self.id as usize].uniform_location_cache[loc_id as usize].as_ref();
        renderer.gl.uniform3f(Some(location), x, y, z);
    }
    /// ------------------------------------------------------------------------
    unsafe fn set_mat4(&self, renderer: &GltfViewerRenderer, loc_id: i32, mat: &Matrix4<f32>) {
        if loc_id == -1 { return; }
        let location = renderer.shaders[self.id as usize].uniform_location_cache[loc_id as usize].as_ref();
        let matrix: [f32; 16] = [mat[0][0],mat[0][1],mat[0][2],mat[0][3],
                             mat[1][0],mat[1][1],mat[1][2],mat[1][3],
                             mat[2][0],mat[2][1],mat[2][2],mat[2][3],
                             mat[3][0],mat[3][1],mat[3][2],mat[3][3]];
        renderer.gl.uniform_matrix4fv_with_f32_array(Some(location), false, &matrix);
    }

    unsafe fn attrib_f32_array(&self, renderer: &GltfViewerRenderer, tag: &str, vertex_data: &[f32], size: i32) {
        let shader_info = &renderer.shaders[self.id as usize];
        let vertex_data_attrib = renderer.gl.get_attrib_location(shader_info.program_id.as_ref(), tag);
        renderer.gl.enable_vertex_attrib_array(vertex_data_attrib as u32);

        write_buffer_f32_data(renderer.gl.as_ref(), &vertex_data[..]);
        
        renderer.gl.vertex_attrib_pointer_with_i32(vertex_data_attrib as u32, size, GL::FLOAT, false, 0, 0);
    }

    /// get uniform location with caching
    unsafe fn uniform_location(&mut self, renderer: &mut GltfViewerRenderer, name: &'static str) -> i32 {
        let mut shader_info = &mut renderer.shaders[self.id as usize];

        if let Some(loc_id) = shader_info.uniform_location_map.get(name) {
            return *loc_id;
        }


        let loc_opt = renderer.gl.get_uniform_location(shader_info.program_id.as_ref(), name);

        if loc_opt.is_none() {
            debug!("uniform '{}' unknown for shader {}", name, self.id);
            return -1;
        }

        let loc_id = shader_info.uniform_location_cache.len() as i32;
        shader_info.uniform_location_cache.push(Rc::new(loc_opt.unwrap()));
        shader_info.uniform_location_map.insert(name, loc_id);
        loc_id
    }
}

// write buffer helper functions
pub fn write_buffer_f32_data(gl: &GL, data: &[f32]) {
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();

    let data_location = data.as_ptr() as u32 / 4;

    let data_array = js_sys::Float32Array::new(&memory_buffer)
        .subarray(data_location, data_location + data.len() as u32);

    let buffer = gl.create_buffer().unwrap();

    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &data_array, GL::STATIC_DRAW);
}

pub fn write_buffer_u32_indices(gl: &GL, indices: &[u32]) {
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();

    let indices_location = indices.as_ptr() as u32 / 4;
    let indices_array = js_sys::Uint32Array::new(&memory_buffer)
        .subarray(indices_location, indices_location + indices.len() as u32);

    let index_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
    gl.buffer_data_with_array_buffer_view(
        GL::ELEMENT_ARRAY_BUFFER,
        &indices_array,
        GL::STATIC_DRAW,
    );
}
