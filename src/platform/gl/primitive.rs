use std::mem::size_of;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;
use std::rc::Rc;

use log::{warn, debug};

use crate::importdata::{ImportData};
use crate::render::math::*;
use crate::render::{Material,Primitive,Root,Vertex};
use crate::shader::{PbrShader,ShaderFlags};

use crate::platform::{GltfViewerRenderer,UniformHelpers};

/// Get offset to struct member, similar to `offset_of` in C/C++
/// From [here](https://stackoverflow.com/questions/40310483/how-to-get-pointer-offset-in-bytes/40310851#40310851)
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(ptr::null() as *const $ty)).$field as *const _ as usize
    }
}


pub trait PrimitiveHelpers {
    unsafe fn setup_primitive(&mut self, g_primitive: &gltf::Primitive<'_>, imp: &ImportData, root: &mut Root, base_path: &Path, renderer: &mut GltfViewerRenderer) -> ShaderFlags ;

    unsafe fn draw(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3, renderer: &GltfViewerRenderer);
    unsafe fn configure_shader(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3, renderer: &GltfViewerRenderer);
}

impl PrimitiveHelpers for Primitive {
    unsafe fn setup_primitive(&mut self, g_primitive: &gltf::Primitive<'_>, imp: &ImportData, root: &mut Root, base_path: &Path, renderer: &mut GltfViewerRenderer) -> ShaderFlags  {
        let buffers = &imp.buffers;
        let reader = g_primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let positions = {
            let iter = reader
                .read_positions()
                .unwrap_or_else(||
                    panic!("primitives must have the POSITION attribute")
                );
            iter.collect::<Vec<_>>()
        };

        let mut shader_flags = ShaderFlags::empty();

        // map position data into vertex objects
        let mut vertices: Vec<Vertex> = positions
            .into_iter()
            .map(|position| {
                Vertex {
                    position: Vector3::from(position),
                    ..Vertex::default()
                }
            }).collect();


        // normals
        if let Some(normals) = reader.read_normals() {
            for (i, normal) in normals.enumerate() {
                vertices[i].normal = Vector3::from(normal);
            }
            shader_flags |= ShaderFlags::HAS_NORMALS;
        }
        else {
            debug!("Found no NORMALs for primitive \
                   (flat normal calculation not implemented yet)");
        }

        // tangents
        if let Some(tangents) = reader.read_tangents() {
            for (i, tangent) in tangents.enumerate() {
                vertices[i].tangent = Vector4::from(tangent);
            }
            shader_flags |= ShaderFlags::HAS_TANGENTS;
        }
        else {
            debug!("Found no TANGENTS for primitive \
                   (tangent calculation not implemented yet)");
        }

        // texture coordinates
        let mut tex_coord_set = 0;
        while let Some(tex_coords) = reader.read_tex_coords(tex_coord_set) {
            if tex_coord_set > 1 {
                warn!("Ignoring texture coordinate set {}, \
                        only supporting 2 sets at the moment",
                        tex_coord_set);
                tex_coord_set += 1;
                continue;
            }
            for (i, tex_coord) in tex_coords.into_f32().enumerate() {
                match tex_coord_set {
                    0 => vertices[i].tex_coord_0 = Vector2::from(tex_coord),
                    1 => vertices[i].tex_coord_1 = Vector2::from(tex_coord),
                    _ => unreachable!()
                }
            }
            shader_flags |= ShaderFlags::HAS_UV;
            tex_coord_set += 1;
        }

        // colors
        if let Some(colors) = reader.read_colors(0) {
            let colors = colors.into_rgba_f32();
            for (i, c) in colors.enumerate() {
                vertices[i].color_0 = c.into();
            }
            shader_flags |= ShaderFlags::HAS_COLORS;
        }
        if reader.read_colors(1).is_some() {
            warn!("Ignoring further color attributes, only supporting COLOR_0.");
        }

        if let Some(joints) = reader.read_joints(0) {
            for (i, joint) in joints.into_u16().enumerate() {
                vertices[i].joints_0 = joint;
            }
        }
        if reader.read_joints(1).is_some() {
            warn!("Ignoring further joint attributes, only supporting JOINTS_0.");
        }

        if let Some(weights) = reader.read_weights(0) {
            for (i, weights) in weights.into_f32().enumerate() {
                vertices[i].weights_0 = weights.into();
            }
        }
        if reader.read_weights(1).is_some() {
            warn!("Ignoring further weight attributes, only supporting WEIGHTS_0.");
        }

        let indices = reader
            .read_indices()
            .map(|read_indices| {
                read_indices.into_u32().collect::<Vec<_>>()
            });
        
        // update self counts
        self.num_vertices = vertices.len() as u32;
        self.num_indices = indices.as_ref().map(|i| i.len()).unwrap_or(0) as u32;

        // material
        let g_material = g_primitive.material();
        
        if let Some(mat) = root.materials.iter().find(|m| (***m).index == g_material.index()) {
            self.material = Rc::clone(mat).into()
        }

        if self.material.is_none() { // no else due to borrow checker madness
            let mat = Rc::new(Material::from_gltf(&g_material, root, imp, base_path, renderer));
            root.materials.push(Rc::clone(&mat));
            self.material = Some(mat);
        };
        let material = self.material.as_ref().unwrap();
        shader_flags |= material.shader_flags();

        let mut new_shader = false; // borrow checker workaround
        self.pbr_shader =
            if let Some(shader) = root.shaders.get(&shader_flags) {
                Rc::clone(shader).into()
            }
            else {
                new_shader = true;
                Rc::new(PbrShader::new(shader_flags, renderer)).into()

            };
        if new_shader {
            root.shaders.insert(shader_flags, Rc::clone(self.pbr_shader.as_ref().unwrap()));
        }
        
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

        shader_flags
    }
    
    /// render the mesh
    unsafe fn draw(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3, renderer: &GltfViewerRenderer) {
        // TODO!: determine if shader+material already active to reduce work...

        if self.material.as_ref().unwrap().double_sided {
            gl::Disable(gl::CULL_FACE);
        } else {
            gl::Enable(gl::CULL_FACE);
        }

        if self.mode == gl::POINTS {
            gl::PointSize(10.0);
        }

        self.configure_shader(model_matrix, mvp_matrix, camera_position, renderer);

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

        if self.material.as_ref().unwrap().alpha_mode != gltf::material::AlphaMode::Opaque {
            let shader = &self.pbr_shader.as_ref().unwrap().shader;

            gl::Disable(gl::BLEND);
            shader.set_float(renderer, self.pbr_shader.as_ref().unwrap().uniforms.u_AlphaBlend, 0.0);
            if self.material.as_ref().unwrap().alpha_mode == gltf::material::AlphaMode::Mask {
                shader.set_float(renderer, self.pbr_shader.as_ref().unwrap().uniforms.u_AlphaCutoff, 0.0);
            }
        }
    }

    unsafe fn configure_shader(&self, model_matrix: &Matrix4,
        mvp_matrix: &Matrix4, camera_position: &Vector3, renderer: &GltfViewerRenderer)
    {
        // let pbr_shader = &Rc::get_mut(&mut self.pbr_shader).unwrap();
        let mat = &self.material.as_ref().unwrap();
        let shader = &self.pbr_shader.as_ref().unwrap().shader;
        let uniforms = &self.pbr_shader.as_ref().unwrap().uniforms;
        self.pbr_shader.as_ref().unwrap().shader.use_program(renderer);

        // camera params
        shader.set_mat4(renderer, uniforms.u_ModelMatrix, model_matrix);
        shader.set_mat4(renderer, uniforms.u_MVPMatrix, mvp_matrix);
        shader.set_vector3(renderer, uniforms.u_Camera, camera_position);

        // alpha blending
        if mat.alpha_mode != gltf::material::AlphaMode::Opaque {
            // BLEND + MASK
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            shader.set_float(renderer, uniforms.u_AlphaBlend, 1.0);

            if mat.alpha_mode == gltf::material::AlphaMode::Mask {
                shader.set_float(renderer, uniforms.u_AlphaCutoff, mat.alpha_cutoff);
            }
        }

        // NOTE: for sampler numbers, see also PbrShader constructor
        shader.set_vector4(renderer, uniforms.u_BaseColorFactor, &mat.base_color_factor);
        if let Some(ref base_color_texture) = mat.base_color_texture {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, base_color_texture.id);
            shader.set_int(renderer, uniforms.u_BaseColorTexCoord, base_color_texture.tex_coord as i32);
        }
        if let Some(ref normal_texture) = mat.normal_texture {
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, normal_texture.id);
            shader.set_int(renderer, uniforms.u_NormalTexCoord, normal_texture.tex_coord as i32);
            shader.set_float(renderer, uniforms.u_NormalScale, mat.normal_scale.unwrap_or(1.0));
        }
        if let Some(ref emissive_texture) = mat.emissive_texture {
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, emissive_texture.id);
            shader.set_int(renderer, uniforms.u_EmissiveTexCoord, emissive_texture.tex_coord as i32);
            shader.set_vector3(renderer, uniforms.u_EmissiveFactor, &mat.emissive_factor);
        }

        if let Some(ref mr_texture) = mat.metallic_roughness_texture {
            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, mr_texture.id);
            shader.set_int(renderer, uniforms.u_MetallicRoughnessTexCoord, mr_texture.tex_coord as i32);
        }
        shader.set_vec2(renderer, uniforms.u_MetallicRoughnessValues,
            mat.metallic_factor, mat.roughness_factor);

        if let Some(ref occlusion_texture) = mat.occlusion_texture {
            gl::ActiveTexture(gl::TEXTURE4);
            gl::BindTexture(gl::TEXTURE_2D, occlusion_texture.id);
            shader.set_int(renderer, uniforms.u_OcclusionTexCoord, occlusion_texture.tex_coord as i32);
            shader.set_float(renderer, uniforms.u_OcclusionStrength, mat.occlusion_strength);
        }
    }
}
