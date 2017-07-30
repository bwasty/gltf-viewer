use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;

use gl;
use gltf;
use gltf::mesh::{ Indices, TexCoords, Colors };
use gltf::json::mesh::Mode;

use render::math::*;
use render::{Material, Scene};
use shader::Shader;

#[repr(C)]
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
    vao: u32,
    vbo: u32,

    ebo: u32,
    num_indices: u32,

    material: Rc<Material>,
    // TODO!: mode, targets
}

impl Primitive {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, material: Rc<Material>) -> Primitive {
        let mut prim = Primitive {
            num_indices: indices.len() as u32,
            vao: 0, vbo: 0, ebo: 0,
            material,
        };

        // now that we have all the required data, set the vertex buffers and its attribute pointers.
        unsafe { prim.setup_primitive(vertices, indices) }
        prim
    }

    pub fn from_gltf(
        g_primitive: gltf::mesh::Primitive,
        primitive_index: usize,
        mesh_index: usize,
        scene: &mut Scene) -> Primitive
    {
        // positions
        let positions = g_primitive.positions()
            .expect(&format!("primitives must have the POSITION attribute (mesh: {}, primitive: {})",
                mesh_index, primitive_index));
        let mut vertices: Vec<Vertex> = positions
            .map(|position| {
                Vertex {
                    position: Vector3::from(position),
                    ..Vertex::default()
                }
            }).collect();

        // normals
        if let Some(normals) = g_primitive.normals() {
            for (i, normal) in normals.enumerate() {
                vertices[i].normal = Vector3::from(normal);
            }
        }
        else {
            println!("WARNING: found no NORMALs for primitive {} of mesh {} \
                      (flat normal calculation not implemented yet)", primitive_index, mesh_index);
        }

        // tangents
        if let Some(tangents) = g_primitive.tangents() {
            for (i, tangent) in tangents.enumerate() {
                vertices[i].tangent = Vector4::from(tangent);
            }
        }

        // texture coordinates
        let mut tex_coord_set = 0;
        while let Some(tex_coords) = g_primitive.tex_coords(tex_coord_set) {
            if tex_coord_set > 1 {
                println!("WARNING: Ignoring texture coordinate set {}, \
                          only supporting 2 sets at the moment. (mesh: {}, primitive: {})",
                          tex_coord_set, mesh_index, primitive_index);
                tex_coord_set += 1;
                continue;
            }
            let tex_coords = match tex_coords {
                TexCoords::F32(tc) => tc,
                // TODO! TexCoords::U8/U16
                TexCoords::U8(_) => unimplemented!(),
                TexCoords::U16(_) => unimplemented!(),
            };
            for (i, tex_coord) in tex_coords.enumerate() {
                match tex_coord_set {
                    0 => vertices[i].tex_coord_0 = Vector2::from(tex_coord),
                    1 => vertices[i].tex_coord_1 = Vector2::from(tex_coord),
                    _ => unreachable!()
                }
            }
            tex_coord_set += 1;
        }

        // colors
        let mut color_set = 0;
        while let Some(colors) = g_primitive.colors(color_set) {
            if color_set > 0 {
                println!("WARNING: Ignoring color set {}, \
                          only supporting 1 set at the moment. (mesh: {}, primitive: {})",
                          color_set, mesh_index, primitive_index);
                color_set += 1;
                continue;
            }
            let colors = match colors {
                // TODO!: support other color formats
                // Colors::RgbU8(Iter<'a, [u8; 3]>),
                // Colors::RgbaU8(Iter<'a, [u8; 4]>),
                // Colors::RgbU16(Iter<'a, [u16; 3]>),
                // Colors::RgbaU16(Iter<'a, [u16; 4]>),
                Colors::RgbF32(c) => c,
                // Colors::RgbaF32(c),
                _ => unimplemented!()
            };
            for (i, color) in colors.enumerate() {
                vertices[i].color_0 = Vector3::from(color);
            }
            color_set += 1;
        }

        let indices = g_primitive.indices()
            .expect("not yet implemented: Indices required at the moment!");

        // convert indices to u32 if necessary
        // TODO?: use indices as they are?
        let indices: Vec<u32> = match indices {
            Indices::U8(indices) => indices.map(|i| i as u32).collect(),
            Indices::U16(indices) => indices.map(|i| i as u32).collect(),
            Indices::U32(indices) => indices.collect(),
        };

        match g_primitive.mode() {
            Mode::Triangles => (),
            _ => panic!("not yet implemented: primitive mode must be Triangles.")
        }

        // TODO!!: unwraps for Triangle, SimpleMeshes, Cameras, AnimatedTriangle
        let g_material = g_primitive.material()
            .unwrap(); // NOTE: tmp - see https://github.com/alteous/gltf/issues/57

        let mut material = None;
        if let Some(mat) = scene.materials.iter().find(|m| (***m).index == g_material.index()) {
            material = mat.clone().into()
        }

        if material.is_none() { // no else due to borrow checker madness
            let mat = Rc::new(Material::from_gltf(&g_material, scene));
            scene.materials.push(mat.clone());
            material = mat.into();
        };

        Primitive::new(vertices, indices, material.unwrap())
    }

    /// render the mesh
    pub unsafe fn draw(&self, shader: &mut Shader) {
        // TODO: fully cache uniform locations
        let loc = shader.uniform_location("base_color_factor");
        shader.set_vector4(loc, &self.material.base_color_factor);
        if let Some(ref base_color_texture) = self.material.base_color_texture {
            let loc = shader.uniform_location("base_color_texture");
            shader.set_int(loc, 0);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, base_color_texture.id);
        }

        // draw mesh
        gl::BindVertexArray(self.vao);
        gl::DrawElements(gl::TRIANGLES, self.num_indices as i32, gl::UNSIGNED_INT, ptr::null());
        gl::BindVertexArray(0);

        // always good practice to set everything back to defaults once configured.
        gl::ActiveTexture(gl::TEXTURE0);
    }

    unsafe fn setup_primitive(&mut self, vertices: Vec<Vertex>, indices: Vec<u32>) {
        // create buffers/arrays
        gl::GenVertexArrays(1, &mut self.vao);
        gl::GenBuffers(1, &mut self.vbo);
        gl::GenBuffers(1, &mut self.ebo);

        gl::BindVertexArray(self.vao);
        // load data into vertex buffers
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        // A great thing about structs with repr(C) is that their memory layout is sequential for all its items.
        // The effect is that we can simply pass a pointer to the struct and it translates perfectly to a glm::vec3/2 array which
        // again translates to 3/2 floats which translates to a byte array.
        let size = (vertices.len() * size_of::<Vertex>()) as isize;
        let data = &vertices[0] as *const Vertex as *const c_void;
        gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
        let size = (indices.len() * size_of::<u32>()) as isize;
        let data = &indices[0] as *const u32 as *const c_void;
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

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
