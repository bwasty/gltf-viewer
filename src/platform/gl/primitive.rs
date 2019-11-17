use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr;

use crate::render::math::*;
use crate::render::{Primitive,Vertex};


use crate::platform::{UniformHelpers};

/// Get offset to struct member, similar to `offset_of` in C/C++
/// From [here](https://stackoverflow.com/questions/40310483/how-to-get-pointer-offset-in-bytes/40310851#40310851)
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(ptr::null() as *const $ty)).$field as *const _ as usize
    }
}


pub trait PrimitiveHelpers {
    unsafe fn draw(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3);
    unsafe fn configure_shader(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3);
    unsafe fn setup_primitive(&mut self, vertices: &[Vertex], indices: Option<Vec<u32>>);
}

impl PrimitiveHelpers for Primitive {
    /// render the mesh
    unsafe fn draw(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3) {
        // TODO!: determine if shader+material already active to reduce work...

        if self.material.double_sided {
            gl::Disable(gl::CULL_FACE);
        } else {
            gl::Enable(gl::CULL_FACE);
        }

        if self.mode == gl::POINTS {
            gl::PointSize(10.0);
        }

        self.configure_shader(model_matrix, mvp_matrix, camera_position);

        // draw mesh
        gl::BindVertexArray(self.vao);
        if self.ebo.is_some() {
            gl::DrawElements(self.mode, self.num_indices as i32, gl::UNSIGNED_INT, ptr::null());
        }
        else {
            gl::DrawArrays(self.mode, 0, self.num_vertices as i32)
        }

        gl::BindVertexArray(0);
        gl::ActiveTexture(gl::TEXTURE0);

        if self.material.alpha_mode != gltf::material::AlphaMode::Opaque {
            let shader = &self.pbr_shader.shader;

            gl::Disable(gl::BLEND);
            shader.set_float(self.pbr_shader.uniforms.u_AlphaBlend, 0.0);
            if self.material.alpha_mode == gltf::material::AlphaMode::Mask {
                shader.set_float(self.pbr_shader.uniforms.u_AlphaCutoff, 0.0);
            }
        }
    }

    unsafe fn configure_shader(&self, model_matrix: &Matrix4,
        mvp_matrix: &Matrix4, camera_position: &Vector3)
    {
        // let pbr_shader = &Rc::get_mut(&mut self.pbr_shader).unwrap();
        let mat = &self.material;
        let shader = &self.pbr_shader.shader;
        let uniforms = &self.pbr_shader.uniforms;
        self.pbr_shader.shader.use_program();

        // camera params
        shader.set_mat4(uniforms.u_ModelMatrix, model_matrix);
        shader.set_mat4(uniforms.u_MVPMatrix, mvp_matrix);
        shader.set_vector3(uniforms.u_Camera, camera_position);

        // alpha blending
        if mat.alpha_mode != gltf::material::AlphaMode::Opaque {
            // BLEND + MASK
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            shader.set_float(uniforms.u_AlphaBlend, 1.0);

            if mat.alpha_mode == gltf::material::AlphaMode::Mask {
                shader.set_float(uniforms.u_AlphaCutoff, mat.alpha_cutoff);
            }
        }

        // NOTE: for sampler numbers, see also PbrShader constructor
        shader.set_vector4(uniforms.u_BaseColorFactor, &mat.base_color_factor);
        if let Some(ref base_color_texture) = mat.base_color_texture {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, base_color_texture.id);
            shader.set_int(uniforms.u_BaseColorTexCoord, base_color_texture.tex_coord as i32);
        }
        if let Some(ref normal_texture) = mat.normal_texture {
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, normal_texture.id);
            shader.set_int(uniforms.u_NormalTexCoord, normal_texture.tex_coord as i32);
            shader.set_float(uniforms.u_NormalScale, mat.normal_scale.unwrap_or(1.0));
        }
        if let Some(ref emissive_texture) = mat.emissive_texture {
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, emissive_texture.id);
            shader.set_int(uniforms.u_EmissiveTexCoord, emissive_texture.tex_coord as i32);
            shader.set_vector3(uniforms.u_EmissiveFactor, &mat.emissive_factor);
        }

        if let Some(ref mr_texture) = mat.metallic_roughness_texture {
            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, mr_texture.id);
            shader.set_int(uniforms.u_MetallicRoughnessTexCoord, mr_texture.tex_coord as i32);
        }
        shader.set_vec2(uniforms.u_MetallicRoughnessValues,
            mat.metallic_factor, mat.roughness_factor);

        if let Some(ref occlusion_texture) = mat.occlusion_texture {
            gl::ActiveTexture(gl::TEXTURE4);
            gl::BindTexture(gl::TEXTURE_2D, occlusion_texture.id);
            shader.set_int(uniforms.u_OcclusionTexCoord, occlusion_texture.tex_coord as i32);
            shader.set_float(uniforms.u_OcclusionStrength, mat.occlusion_strength);
        }
    }

    unsafe fn setup_primitive(&mut self, vertices: &[Vertex], indices: Option<Vec<u32>>) {
        // create buffers/arrays
        gl::GenVertexArrays(1, &mut self.vao);
        gl::GenBuffers(1, &mut self.vbo);
        if indices.is_some() {
            let mut ebo = 0;
            gl::GenBuffers(1, &mut ebo);
            self.ebo = Some(ebo);
        }

        gl::BindVertexArray(self.vao);
        // load data into vertex buffers
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        let size = (vertices.len() * size_of::<Vertex>()) as isize;
        let data = &vertices[0] as *const Vertex as *const c_void;
        gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        if let Some(ebo) = self.ebo {
            let indices = indices.unwrap();
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            let size = (indices.len() * size_of::<u32>()) as isize;
            let data = &indices[0] as *const u32 as *const c_void;
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, data, gl::STATIC_DRAW);
        }

        // set the vertex attribute pointers
        let size = size_of::<Vertex>() as i32;
        // POSITION
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, position) as *const c_void);
        // NORMAL
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, normal) as *const c_void);
        // TANGENT
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, tangent) as *const c_void);
        // TEXCOORD_0
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, tex_coord_0) as *const c_void);
        // TEXCOORD_1
        gl::EnableVertexAttribArray(4);
        gl::VertexAttribPointer(4, 2, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, tex_coord_1) as *const c_void);
        // COLOR_0
        gl::EnableVertexAttribArray(5);
        gl::VertexAttribPointer(5, 4, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, color_0) as *const c_void);
        // JOINTS_0
        gl::EnableVertexAttribArray(6);
        // TODO: normalization?
        gl::VertexAttribPointer(6, 4, gl::UNSIGNED_SHORT, gl::FALSE, size, offset_of!(Vertex, joints_0) as *const c_void);
        // WEIGHTS_0
        gl::EnableVertexAttribArray(7);
        gl::VertexAttribPointer(7, 4, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, weights_0) as *const c_void);

        gl::BindVertexArray(0);
    }
}
