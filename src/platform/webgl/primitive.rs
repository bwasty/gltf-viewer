use std::path::Path;
use std::rc::Rc;
use web_sys::WebGl2RenderingContext as GL;

use crate::{debug,warn};
use crate::importdata::{ImportData};
use crate::render::math::*;
use crate::render::{Material,Primitive,Root};
use crate::shader::{PbrShader,ShaderFlags};


use crate::platform::{GltfViewerRenderer,UniformHelpers,write_buffer_u32_indices};

pub trait PrimitiveHelpers {
    unsafe fn setup_primitive(&mut self, g_primitive: &gltf::Primitive<'_>, imp: &ImportData, root: &mut Root, base_path: &Path, renderer: &mut GltfViewerRenderer) -> ShaderFlags ;

    unsafe fn draw(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3, renderer: &GltfViewerRenderer);
}

impl PrimitiveHelpers for Primitive {
    /// Buffers primitive into renderer context
    unsafe fn setup_primitive(&mut self, g_primitive: &gltf::Primitive<'_>, imp: &ImportData, root: &mut Root, base_path: &Path, renderer: &mut GltfViewerRenderer) -> ShaderFlags {
        let glr = Rc::clone(&renderer.gl);
        let gl = glr.as_ref();
        let mut shader_flags = ShaderFlags::empty();

        // load data
        let buffers = &imp.buffers;
        let reader = g_primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let positions = {
            let iter = reader
                .read_positions()
                .unwrap_or_else(||
                    panic!("primitives must have the POSITION attribute")
                );
           iter.flat_map(|a|a.iter().map(|i|*i).collect::<Vec<f32>>()).collect::<Vec<_>>()
        };

        // normals
        let mut normals = None;
        if let Some(normals_reader) = reader.read_normals() {
            normals = Some(normals_reader.flat_map(|a|a.iter().map(|i|*i).collect::<Vec<f32>>()).collect::<Vec<_>>());
            shader_flags |= ShaderFlags::HAS_NORMALS;
        }
        else {
            debug!("Found no NORMALs for primitive \
                   (flat normal calculation not implemented yet)");
        }

        // tangents
        let mut tangents = None;
        if let Some(tangents_reader) = reader.read_tangents() {
            tangents = Some(tangents_reader.flat_map(|a|a.iter().map(|i|*i).collect::<Vec<f32>>()).collect::<Vec<_>>());
            shader_flags |= ShaderFlags::HAS_TANGENTS;
        }
        else {
            debug!("Found no TANGENTS for primitive \
                   (tangent calculation not implemented yet)");
        }

        // texture coordinates
        let mut tex_coord_set = 0;
        let mut tex_coord_0 = None;
        let mut tex_coord_1 = None;
        while let Some(tex_coords) = reader.read_tex_coords(tex_coord_set) {
            if tex_coord_set > 1 {
                warn!("Ignoring texture coordinate set {}, \
                        only supporting 2 sets at the moment. ",
                        tex_coord_set);
                tex_coord_set += 1;
                continue;
            }
            match tex_coord_set {
                0 => tex_coord_0 = Some(tex_coords.into_f32().flat_map(|a|a.iter().map(|i|*i).collect::<Vec<f32>>()).collect::<Vec<_>>()),
                1 => tex_coord_1 = Some(tex_coords.into_f32().flat_map(|a|a.iter().map(|i|*i).collect::<Vec<f32>>()).collect::<Vec<_>>()),
                _ => unreachable!()
            }
            shader_flags |= ShaderFlags::HAS_UV;
            tex_coord_set += 1;
        }

        // colors
        let mut colors = None;
        if let Some(colors_reader) = reader.read_colors(0) {
            colors = Some(colors_reader.into_rgba_f32().flat_map(|a|a.iter().map(|i|*i).collect::<Vec<f32>>()).collect::<Vec<_>>());
            shader_flags |= ShaderFlags::HAS_COLORS;
        }
        if reader.read_colors(1).is_some() {
            warn!("Ignoring further color attributes, only supporting COLOR_0.");
        }

        // joints
        // let mut joints = None;
        // if let Some(joints_reader) = reader.read_joints(0) {
        //     joints = Some(joints_reader.into_u16().flat_map(|a|a.iter().map(|i|*i).collect::<Vec<u16>>()).collect::<Vec<_>>());
        // }
        // if reader.read_joints(1).is_some() {
        //     warn!("Ignoring further joint attributes, only supporting JOINTS_0.");
        // }

        // weights
        // let mut weights = None;
        // if let Some(weights_reader) = reader.read_weights(0) {
        //     weights = Some(weights_reader.into_f32().flat_map(|a|a.iter().map(|i|*i).collect::<Vec<f32>>()).collect::<Vec<_>>());
        // }
        // if reader.read_weights(1).is_some() {
        //     warn!("Ignoring further weight attributes, only supporting WEIGHTS_0.");
        // }

        // indices
        let indices = reader
            .read_indices()
            .map(|read_indices| {
                read_indices.into_u32().collect::<Vec<_>>()
            });

        // update self counts
        self.num_vertices = positions.len() as u32 / 3;
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

        // reborrow shader
        let shader = &self.pbr_shader.as_ref().unwrap().shader;


        // create position and index buffers
        let vbo_obj = Rc::new(gl.create_buffer().expect("Failed to create vbo"));
        self.vbo = renderer.buffers.len() as u32;
        renderer.buffers.push(vbo_obj);

        if !indices.is_none() {
            let ebo_obj = Rc::new(gl.create_buffer().expect("Failed to create ebo"));
            self.ebo = Some(renderer.buffers.len() as u32);
            renderer.buffers.push(ebo_obj);
        };
        
        // create vertex array object
        let vao_obj = Rc::new(gl.create_vertex_array().expect("Failed to create vao"));
        self.vao = renderer.vaos.len() as u32;
        renderer.vaos.push(vao_obj);
        gl.bind_vertex_array(Some(renderer.vaos[self.vao as usize].as_ref()));

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(renderer.buffers[self.vbo as usize].as_ref()));

        // buffer data
        shader.attrib_f32_array(renderer, "a_Position", positions.as_slice(), 3);
        if let Some(normals) = &normals {
            shader.attrib_f32_array(renderer, "a_Normal", normals.as_slice(), 3);
        }
        if let Some(tangents) = &tangents {
            shader.attrib_f32_array(renderer, "a_Tangent", tangents.as_slice(), 4);
        }
        if let Some(tex_coord_0) = &tex_coord_0 {
            shader.attrib_f32_array(renderer, "a_UV_0", tex_coord_0.as_slice(), 2);
        }
        if let Some(tex_coord_1) = &tex_coord_1 {
            shader.attrib_f32_array(renderer, "a_UV_1", tex_coord_1.as_slice(), 2);
        }
        if let Some(colors) = &colors {
            shader.attrib_f32_array(renderer, "a_Color", colors.as_slice(), 4);
        }
        // if let Some(joints) = &joints {
        //     shader.write_attrib_data(renderer, "a_Joints", joints.as_slice(), 4);
        // }
        // if let Some(weights) = &weights {
        //     shader.write_attrib_data(renderer, "a_Weights", weights.as_slice(), 4);
        // }

        // element indices
        if let Some(indices) = &indices {
            gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(renderer.buffers[self.ebo.unwrap() as usize].as_ref()));
            write_buffer_u32_indices(gl, indices.as_slice());
        }

        shader_flags
    }
    
    // draws primitive
    unsafe fn draw(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3, renderer: &GltfViewerRenderer) {
        let glr = Rc::clone(&renderer.gl);
        let gl = glr.as_ref();
        
        if self.material.as_ref().unwrap().double_sided {
            gl.disable(GL::CULL_FACE);
        } else {
            gl.enable(GL::CULL_FACE);
        }

        if self.mode == GL::POINTS {
            // TODO not supported in webgl
            // gl.point_size(10.0);
        }

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
            gl.enable(GL::BLEND);
            gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
            shader.set_float(renderer, uniforms.u_AlphaBlend, 1.0);

            if mat.alpha_mode == gltf::material::AlphaMode::Mask {
                shader.set_float(renderer, uniforms.u_AlphaCutoff, mat.alpha_cutoff);
            }
        }

        // NOTE: for sampler numbers, see also PbrShader constructor
        shader.set_vector4(renderer, uniforms.u_BaseColorFactor, &mat.base_color_factor);
        if let Some(ref base_color_texture) = mat.base_color_texture {
            gl.active_texture(GL::TEXTURE0);
            let base_color_texture_id = renderer.textures[base_color_texture.id as usize].as_ref();
            gl.bind_texture(GL::TEXTURE_2D, Some(base_color_texture_id));
            shader.set_int(renderer, uniforms.u_BaseColorTexCoord, base_color_texture.tex_coord as i32);
        }
        if let Some(ref normal_texture) = mat.normal_texture {
            gl.active_texture(GL::TEXTURE1);
            let normal_texture_texture_id = renderer.textures[normal_texture.id as usize].as_ref();
            gl.bind_texture(GL::TEXTURE_2D, Some(normal_texture_texture_id));
            shader.set_int(renderer, uniforms.u_NormalTexCoord, normal_texture.tex_coord as i32);
            shader.set_float(renderer, uniforms.u_NormalScale, mat.normal_scale.unwrap_or(1.0));
        }
        if let Some(ref emissive_texture) = mat.emissive_texture {
            gl.active_texture(GL::TEXTURE2);
            let emissive_texture_texture_id = renderer.textures[emissive_texture.id as usize].as_ref();
            gl.bind_texture(GL::TEXTURE_2D, Some(emissive_texture_texture_id));
            shader.set_int(renderer, uniforms.u_EmissiveTexCoord, emissive_texture.tex_coord as i32);
            shader.set_vector3(renderer, uniforms.u_EmissiveFactor, &mat.emissive_factor);
        }

        if let Some(ref mr_texture) = mat.metallic_roughness_texture {
            gl.active_texture(GL::TEXTURE3);
            let mr_texture_texture_id = renderer.textures[mr_texture.id as usize].as_ref();
            gl.bind_texture(GL::TEXTURE_2D, Some(mr_texture_texture_id));
            shader.set_int(renderer, uniforms.u_MetallicRoughnessTexCoord, mr_texture.tex_coord as i32);
        }
        shader.set_vec2(renderer, uniforms.u_MetallicRoughnessValues,
            mat.metallic_factor, mat.roughness_factor);

        if let Some(ref occlusion_texture) = mat.occlusion_texture {
            gl.active_texture(GL::TEXTURE4);
            let occlusion_texture_texture_id = renderer.textures[occlusion_texture.id as usize].as_ref();
            gl.bind_texture(GL::TEXTURE_2D, Some(occlusion_texture_texture_id));
            shader.set_int(renderer, uniforms.u_OcclusionTexCoord, occlusion_texture.tex_coord as i32);
            shader.set_float(renderer, uniforms.u_OcclusionStrength, mat.occlusion_strength);
        }


        // draw mesh
        gl.bind_vertex_array(Some(renderer.vaos[self.vao as usize].as_ref()));
        if self.ebo.is_some() {
            gl.draw_elements_with_i32(self.mode, self.num_indices as i32, GL::UNSIGNED_INT, 0);
        }
        else {
            gl.draw_arrays(self.mode, 0, self.num_vertices as i32);
        }

        gl.bind_vertex_array(None);
        gl.active_texture(GL::TEXTURE0);

        if self.material.as_ref().unwrap().alpha_mode != gltf::material::AlphaMode::Opaque {
            let shader = &self.pbr_shader.as_ref().unwrap().shader;

            gl.disable(GL::BLEND);
            shader.set_float(renderer, self.pbr_shader.as_ref().unwrap().uniforms.u_AlphaBlend, 0.0);
            if self.material.as_ref().unwrap().alpha_mode == gltf::material::AlphaMode::Mask {
                shader.set_float(renderer, self.pbr_shader.as_ref().unwrap().uniforms.u_AlphaCutoff, 0.0);
            }
        }
    }
}
