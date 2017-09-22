use std::mem::size_of;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;
use std::rc::Rc;

use gl;
use gltf;
use gltf::json::mesh::Mode;
use gltf_importer;
use gltf_utils::PrimitiveIterators;

// use camera::Camera;
use render::math::*;
use render::{Material, Scene};
use shader::*;

#[derive(Debug)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub tangent: Vector4,
    pub tex_coord_0: Vector2,
    pub tex_coord_1: Vector2,
    pub color_0: Vector3, // TODO: vec4 support
    // TODO: joints, weights
    // pub joints_0: Vector4,
    // pub weights_0: Vector4,
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vector3::zero(),
            normal: Vector3::zero(),
            tangent: Vector4::zero(),
            tex_coord_0: Vector2::zero(),
            tex_coord_1: Vector2::zero(),
            color_0: Vector3::zero(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    pub id: u32,
    pub type_: String,
    pub path: String,
}

pub struct Primitive {
    pub bounds: Bounds,

    vao: u32,
    vbo: u32,
    num_vertices: u32,

    ebo: Option<u32>,
    num_indices: u32,

    material: Rc<Material>,

    pbr_shader: Rc<PbrShader>,

    // TODO!: mode, targets
}

impl Primitive {
    pub fn new(
        bounds: Bounds,
        vertices: Vec<Vertex>,
        indices: Option<Vec<u32>>,
        material: Rc<Material>,
        shader: Rc<PbrShader>,
    ) -> Primitive {
        let num_indices = indices.as_ref().map(|i| i.len()).unwrap_or(0);
        let mut prim = Primitive {
            bounds,
            num_vertices: vertices.len() as u32,
            num_indices: num_indices as u32,
            vao: 0, vbo: 0, ebo: None,
            material,
            pbr_shader: shader,
        };

        // now that we have all the required data, set the vertex buffers and its attribute pointers.
        unsafe { prim.setup_primitive(vertices, indices) }
        prim
    }

    pub fn from_gltf(
        g_primitive: gltf::Primitive,
        primitive_index: usize,
        mesh_index: usize,
        scene: &mut Scene,
        buffers: &gltf_importer::Buffers,
        base_path: &Path) -> Primitive
    {
        // positions
        let positions = g_primitive.positions(buffers)
            .expect(&format!("primitives must have the POSITION attribute (mesh: {}, primitive: {})",
                mesh_index, primitive_index));

        let bounds = g_primitive.position_bounds()
            .unwrap(); // can't fail if validated "minimally"
        let mut vertices: Vec<Vertex> = positions
            .map(|position| {
                Vertex {
                    position: Vector3::from(position),
                    ..Vertex::default()
                }
            }).collect();

        let mut shader_flags = ShaderFlags::empty();

        // normals
        if let Some(normals) = g_primitive.normals(buffers) {
            for (i, normal) in normals.enumerate() {
                vertices[i].normal = Vector3::from(normal);
            }
            shader_flags |= HAS_NORMALS;
        }
        else {
            debug!("Found no NORMALs for primitive {} of mesh {} \
                   (flat normal calculation not implemented yet)", primitive_index, mesh_index);
        }

        // tangents
        if let Some(tangents) = g_primitive.tangents(buffers) {
            for (i, tangent) in tangents.enumerate() {
                vertices[i].tangent = Vector4::from(tangent);
            }
            shader_flags |= HAS_TANGENTS;
        }
        else {
            debug!("Found no TANGENTS for primitive {} of mesh {} \
                   (tangent calculation not implemented yet)", primitive_index, mesh_index);
        }

        // texture coordinates
        let mut tex_coord_set = 0;
        while let Some(tex_coords) = g_primitive.tex_coords_f32(tex_coord_set, buffers) {
            if tex_coord_set > 1 {
                warn!("Ignoring texture coordinate set {}, \
                        only supporting 2 sets at the moment. (mesh: {}, primitive: {})",
                        tex_coord_set, mesh_index, primitive_index);
                tex_coord_set += 1;
                continue;
            }
            for (i, tex_coord) in tex_coords.enumerate() {
                match tex_coord_set {
                    0 => vertices[i].tex_coord_0 = Vector2::from(tex_coord),
                    1 => vertices[i].tex_coord_1 = Vector2::from(tex_coord),
                    _ => unreachable!()
                }
            }
            shader_flags |= HAS_UV;
            tex_coord_set += 1;
        }

        // colors
        let mut color_set = 0;
        while let Some(colors) = g_primitive.colors_rgba_f32(color_set, 1.0, buffers) {
            if color_set > 0 {
                warn!("Ignoring color set {}, \
                       only supporting 1 set at the moment. (mesh: {}, primitive: {})",
                       color_set, mesh_index, primitive_index);
                color_set += 1;
                continue;
            }
            // TODO!!: alpha (color attribute)
            for (i, c) in colors.enumerate() {
                vertices[i].color_0 = vec3(c[0], c[1], c[2]);
            }
            shader_flags |= HAS_COLORS;
            color_set += 1;
        }

        let indices: Option<Vec<u32>> = g_primitive.indices_u32(buffers).map(|indices| indices.collect());

        assert_eq!(g_primitive.mode(), Mode::Triangles, "not yet implemented: primitive mode must be Triangles.");

        let g_material = g_primitive.material();

        let mut material = None;
        if let Some(mat) = scene.materials.iter().find(|m| (***m).index == g_material.index()) {
            material = mat.clone().into()
        }

        if material.is_none() { // no else due to borrow checker madness
            let mat = Rc::new(Material::from_gltf(&g_material, scene, buffers, base_path));
            scene.materials.push(mat.clone());
            material = Some(mat);
        };
        let material = material.unwrap();
        shader_flags |= material.shader_flags();

        let mut new_shader = false; // borrow checker workaround
        let shader =
            if let Some(shader) = scene.shaders.get(&shader_flags) {
                shader.clone()
            }
            else {
                new_shader = true;
                PbrShader::new(shader_flags).into()

            };
        if new_shader {
            scene.shaders.insert(shader_flags, shader.clone());
        }

        Primitive::new(bounds.into(), vertices, indices, material, shader)
    }

    /// render the mesh
    pub unsafe fn draw(&self, model_matrix: &Matrix4, mvp_matrix: &Matrix4, camera_position: &Vector3) {
        // TODO!!: determine if shader+material already active to reduce work...

        self.configure_shader(model_matrix, mvp_matrix, camera_position);

        // draw mesh
        gl::BindVertexArray(self.vao);
        if self.ebo.is_some() {
            gl::DrawElements(gl::TRIANGLES, self.num_indices as i32, gl::UNSIGNED_INT, ptr::null());
        }
        else {
            gl::DrawArrays(gl::TRIANGLES, 0, self.num_vertices as i32)
        }

        gl::BindVertexArray(0);
        gl::ActiveTexture(gl::TEXTURE0);
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

        // NOTE: for sampler numbers, see also PbrShader constructor
        shader.set_vector4(uniforms.u_BaseColorFactor, &mat.base_color_factor);
        if let Some(ref base_color_texture) = mat.base_color_texture {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, base_color_texture.id);
        }
        if let Some(ref normal_texture) = mat.normal_texture {
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, normal_texture.id);
        }
        if let Some(ref emissive_texture) = mat.emissive_texture {
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, emissive_texture.id);

            shader.set_vector3(uniforms.u_EmissiveFactor, &mat.emissive_factor);
        }

        if let Some(ref mr_texture) = mat.metallic_roughness_texture {
            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, mr_texture.id);
        }
        shader.set_vec2(uniforms.u_MetallicRoughnessValues,
            mat.metallic_factor, mat.roughness_factor);

        if let Some(ref occlusion_texture) = mat.occlusion_texture {
            gl::ActiveTexture(gl::TEXTURE4);
            gl::BindTexture(gl::TEXTURE_2D, occlusion_texture.id);

            shader.set_float(uniforms.u_OcclusionSampler, mat.occlusion_strength);
        }
    }

    unsafe fn setup_primitive(&mut self, vertices: Vec<Vertex>, indices: Option<Vec<u32>>) {
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
        gl::VertexAttribPointer(5, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, color_0) as *const c_void);

        gl::BindVertexArray(0);
    }
}
