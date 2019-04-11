use std::mem::size_of;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;
use std::rc::Rc;

use gltf;

use yage::gl::{GL, GlFunctions, glenum, objects::{VertexArray, Buffer}};

use log::{warn, debug};

use crate::render::math::*;
use crate::render::{Material, Root};
use crate::shader::*;
use crate::importdata::ImportData;

#[derive(Debug)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub tangent: Vector4,
    pub tex_coord_0: Vector2,
    pub tex_coord_1: Vector2,
    pub color_0: Vector4,
    pub joints_0: [u16; 4],
    pub weights_0: Vector4,
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vector3::zero(),
            normal: Vector3::zero(),
            tangent: Vector4::zero(),
            tex_coord_0: Vector2::zero(),
            tex_coord_1: Vector2::zero(),
            color_0: Vector4::zero(),
            joints_0: [0; 4],
            weights_0: Vector4::zero(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    pub id: u32,
    pub type_: String,
    pub path: String,
}

pub struct Primitive<'a> {
    pub bounds: Aabb3,

    gl: &'a GL,

    vao: VertexArray<'a>,
    vbo: Buffer<'a>,
    num_vertices: u32,

    ebo: Option<Buffer<'a>>,
    num_indices: u32,

    mode: u32,

    material: Rc<Material>,

    pbr_shader: Rc<PbrShader>,

    // TODO!: mode, targets
}

impl<'a> Primitive<'a> {
    pub fn new(
        gl: &'a GL,
        bounds: Aabb3,
        vertices: &[Vertex],
        indices: Option<Vec<u32>>,
        mode: u32,
        material: Rc<Material>,
        shader: Rc<PbrShader>,
    ) -> Primitive<'a> {
        let num_indices = indices.as_ref().map(|i| i.len()).unwrap_or(0);
        let (vao, vbo, ebo) = unsafe { Self::setup_primitive(gl, vertices, indices) };
        Primitive {
            bounds,
            gl,
            num_vertices: vertices.len() as u32,
            num_indices: num_indices as u32,
            vao, vbo, ebo,
            mode,
            material,
            pbr_shader: shader,
        }
    }

    pub fn from_gltf(
        gl: &'a GL,
        g_primitive: &gltf::Primitive<'_>,
        primitive_index: usize,
        mesh_index: usize,
        root: &mut Root,
        imp: &ImportData,
        base_path: &Path) -> Primitive<'a>
    {
        let buffers = &imp.buffers;
        let reader = g_primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let positions = {
            let iter = reader
                .read_positions()
                .unwrap_or_else(||
                    panic!("primitives must have the POSITION attribute (mesh: {}, primitive: {})",
                        mesh_index, primitive_index)
                );
            iter.collect::<Vec<_>>()
        };

        let bounds = g_primitive.bounding_box();
        let bounds = Aabb3 {
            min: bounds.min.into(),
            max: bounds.max.into()
        };

        let mut vertices: Vec<Vertex> = positions
            .into_iter()
            .map(|position| {
                Vertex {
                    position: Vector3::from(position),
                    ..Vertex::default()
                }
            }).collect();

        let mut shader_flags = ShaderFlags::empty();

        // normals
        if let Some(normals) = reader.read_normals() {
            for (i, normal) in normals.enumerate() {
                vertices[i].normal = Vector3::from(normal);
            }
            shader_flags |= ShaderFlags::HAS_NORMALS;
        }
        else {
            debug!("Found no NORMALs for primitive {} of mesh {} \
                   (flat normal calculation not implemented yet)", primitive_index, mesh_index);
        }

        // tangents
        if let Some(tangents) = reader.read_tangents() {
            for (i, tangent) in tangents.enumerate() {
                vertices[i].tangent = Vector4::from(tangent);
            }
            shader_flags |= ShaderFlags::HAS_TANGENTS;
        }
        else {
            debug!("Found no TANGENTS for primitive {} of mesh {} \
                   (tangent calculation not implemented yet)", primitive_index, mesh_index);
        }

        // texture coordinates
        let mut tex_coord_set = 0;
        while let Some(tex_coords) = reader.read_tex_coords(tex_coord_set) {
            if tex_coord_set > 1 {
                warn!("Ignoring texture coordinate set {}, \
                        only supporting 2 sets at the moment. (mesh: {}, primitive: {})",
                        tex_coord_set, mesh_index, primitive_index);
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
            warn!("Ignoring further color attributes, only supporting COLOR_0. (mesh: {}, primitive: {})",
                mesh_index, primitive_index);
        }

        if let Some(joints) = reader.read_joints(0) {
            for (i, joint) in joints.into_u16().enumerate() {
                vertices[i].joints_0 = joint;
            }
        }
        if reader.read_joints(1).is_some() {
            warn!("Ignoring further joint attributes, only supporting JOINTS_0. (mesh: {}, primitive: {})",
                mesh_index, primitive_index);
        }

        if let Some(weights) = reader.read_weights(0) {
            for (i, weights) in weights.into_f32().enumerate() {
                vertices[i].weights_0 = weights.into();
            }
        }
        if reader.read_weights(1).is_some() {
            warn!("Ignoring further weight attributes, only supporting WEIGHTS_0. (mesh: {}, primitive: {})",
                mesh_index, primitive_index);
        }

        let indices = reader
            .read_indices()
            .map(|read_indices| {
                read_indices.into_u32().collect::<Vec<_>>()
            });

        // TODO: spec:
        // Implementation note: When the 'mode' property is set to a non-triangular type
        //(such as POINTS or LINES) some additional considerations must be taken while
        //considering the proper rendering technique:
        //   For LINES with NORMAL and TANGENT properties can render with standard lighting including normal maps.
        //   For all POINTS or LINES with no TANGENT property, render with standard lighting but ignore any normal maps on the material.
        //   For POINTS or LINES with no NORMAL property, don't calculate lighting and instead output the COLOR value for each pixel drawn.
        let mode = g_primitive.mode().as_gl_enum();

        let g_material = g_primitive.material();

        let mut material = None;
        if let Some(mat) = root.materials.iter().find(|m| (***m).index == g_material.index()) {
            material = Rc::clone(mat).into()
        }

        if material.is_none() { // no else due to borrow checker madness
            let mat = Rc::new(Material::from_gltf(&g_material, root, imp, base_path));
            root.materials.push(Rc::clone(&mat));
            material = Some(mat);
        };
        let material = material.unwrap();
        shader_flags |= material.shader_flags();

        let mut new_shader = false; // borrow checker workaround
        let shader =
            if let Some(shader) = root.shaders.get(&shader_flags) {
                Rc::clone(shader)
            }
            else {
                new_shader = true;
                PbrShader::new(shader_flags).into()

            };
        if new_shader {
            root.shaders.insert(shader_flags, Rc::clone(&shader));
        }

        Primitive::new(gl, bounds, &vertices, indices, mode, material, shader)
    }

    /// render the mesh
    pub unsafe fn draw(&self, gl: &GL, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3) {
        // TODO!: determine if shader+material already active to reduce work...

        if self.material.double_sided {
            gl.disable(glenum::Culling::CullFace as _);
        } else {
            gl.enable(glenum::Culling::CullFace as _);
        }

        if self.mode == gl::POINTS {
            gl.point_size(10.0);
        }

        self.configure_shader(model_matrix, mvp_matrix, camera_position);

        // draw mesh
        self.vao.bind();
        if self.ebo.is_some() {
            gl.draw_elements(self.mode, self.num_indices as i32, glenum::DataType::U32 as _, 0);
            // gl::DrawElements(self.mode, self.num_indices as i32, gl::UNSIGNED_INT, ptr::null());
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

    unsafe fn setup_primitive(
        gl: &'a GL,
        vertices: &[Vertex],
        indices: Option<Vec<u32>>
    ) -> (VertexArray<'a>, Buffer<'a>, Option<Buffer<'a>>) {
        // create buffers/arrays
        let vao = VertexArray::new(gl);
        let vbo = Buffer::new(gl, glenum::BufferKind::Array as _);
        let ebo = if indices.is_some() {
            Some(Buffer::new(gl, glenum::BufferKind::ElementArray as _))
        } else {
            None
        };

        vao.bind();
        vbo.bind();
        vbo.set_data(vertices, glenum::DrawMode::Static as _);

        if let Some(ebo) = ebo {
            ebo.bind();
            ebo.set_data(&indices.unwrap(), glenum::DrawMode::Static as _);
        }

        // TODO!!!: attrib_enable...
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

        (vao, vbo, ebo)
    }
}
