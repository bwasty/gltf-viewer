use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr;

use gl;
use gltf;
use gltf::mesh::{ Indices, TexCoord, Color };

use render::math::*;
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
    // TODO!
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
    /*  Mesh Data  */
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,
    pub vao: u32,

    /*  Render data  */
    vbo: u32,
    ebo: u32,

    // TODO: material, mode, targets
}

impl Primitive {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Primitive {
        let mut prim = Primitive {
            vertices, indices, textures,
            vao: 0, vbo: 0, ebo: 0
        };

        // now that we have all the required data, set the vertex buffers and its attribute pointers.
        unsafe { prim.setup_primitive() }
        prim
    }

    pub fn from_gltf(g_primitive: gltf::mesh::Primitive) -> Primitive {
        let positions = g_primitive.position().unwrap();
        let normals = g_primitive.normal()
            .expect("NotImplementedYet: Normals required! Calculation of flat normals not implemented yet.");
        let mut tangents = g_primitive.tangent();

        // TODO!: support the different texcoord and color formats
        let mut tex_coords_0 = match g_primitive.tex_coord(0) {
            Some(tex_coords_0) => {
                Some(match tex_coords_0 {
                    TexCoord::F32(tc) => tc,
                    TexCoord::U8(_) => unimplemented!(),
                    TexCoord::U16(_) => unimplemented!(),
                })
            },
            None => None
        };
        let mut tex_coords_1 = match g_primitive.tex_coord(1) {
            Some(tex_coords_1) => {
                Some(match tex_coords_1 {
                    TexCoord::F32(tc) => tc,
                    TexCoord::U8(_) => unimplemented!(),
                    TexCoord::U16(_) => unimplemented!(),
                })
            },
            None => None
        };
        let mut colors_0 = match g_primitive.color(0) {
            Some(colors_0) => {
                Some(match colors_0 {
                    // Color::RgbU8(Iter<'a, [u8; 3]>),
                    // Color::RgbaU8(Iter<'a, [u8; 4]>),
                    // Color::RgbU16(Iter<'a, [u16; 3]>),
                    // Color::RgbaU16(Iter<'a, [u16; 4]>),
                    Color::RgbF32(c) => c,
                    // Color::RgbaF32(c),
                    _ => unimplemented!()
                })
            }
            None => None
        };

        let indices = g_primitive.indices().expect("NotImplementedYet: Indices required at the moment!");

        let vertices: Vec<Vertex> = positions
            .zip(normals)
            .map(|(position, normal)| {
                let tangent = match tangents {
                    Some(ref mut tangents) => Vector4::from(tangents.next()
                        .expect("Not enough tangents! Expected 1 per vertex.")),
                    None => Vector4::zero()
                };
                let tex_coord_0 = match tex_coords_0 {
                    Some(ref mut tex_coord_0) => {
                        Vector2::from(tex_coord_0.next()
                        .expect("Not enough tex_coords_0! Expected 1 per vertex."))
                    }
                    None => Vector2::zero()
                };
                let tex_coord_1 = match tex_coords_1 {
                    Some(ref mut tex_coord_1) => Vector2::from(tex_coord_1.next()
                        .expect("Not enough tex_coords_1! Expected 1 per vertex.")),
                    None => Vector2::zero()
                };

                let color_0 = match colors_0 {
                    Some(ref mut colors_0) => Vector3::from(colors_0.next()
                        .expect("Not enough color_0 entries! Expected 1 per vertex.")),
                    None => Vector3::zero()
                };
                Vertex {
                    position: Vector3::from(position),
                    normal: Vector3::from(normal),
                    tangent: tangent,
                    tex_coord_0: tex_coord_0,
                    tex_coord_1: tex_coord_1,
                    color_0: color_0,
                    ..Vertex::default()
                }
            })
            .collect();

        // convert indices to u32 if necessary
        // TODO?: use indices as they are?
        let indices: Vec<u32> = match indices {
            Indices::U8(indices) => indices.map(|i| i as u32).collect(),
            Indices::U16(indices) => indices.map(|i| i as u32).collect(),
            Indices::U32(indices) => indices.collect(),
        };

        // TODO: No debug
        // assert_eq!(primitive.mode(), Mode::Triangles);

        // TODO!!: textures
        let textures = Vec::new();
        Primitive::new(vertices, indices, textures)
    }

    /// render the mesh
    pub unsafe fn draw(&self, shader: &Shader) {
        // bind appropriate textures
        let mut diffuse_nr  = 0;
        let mut specular_nr = 0;
        let mut normal_nr   = 0;
        let mut height_nr   = 0;
        for (i, texture) in self.textures.iter().enumerate() {
            gl::ActiveTexture(gl::TEXTURE0 + i as u32); // active proper texture unit before binding
            // retrieve texture number (the N in diffuse_textureN)
            let name = &texture.type_;
            let number = match name.as_str() {
                "texture_diffuse" => {
                    diffuse_nr += 1;
                    diffuse_nr
                },
                "texture_specular" => {
                    specular_nr += 1;
                    specular_nr
                }
                "texture_normal" => {
                    normal_nr += 1;
                    normal_nr
                }
                "texture_height" => {
                    height_nr += 1;
                    height_nr
                }
                _ => panic!("unknown texture type")
            };
            // now set the sampler to the correct texture unit
            let sampler = CString::new(format!("{}{}", name, number)).unwrap();
            gl::Uniform1i(gl::GetUniformLocation(shader.id, sampler.as_ptr()), i as i32);
            // and finally bind the texture
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
        }

        // draw mesh
        gl::BindVertexArray(self.vao);
        gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());
        gl::BindVertexArray(0);

        // always good practice to set everything back to defaults once configured.
        gl::ActiveTexture(gl::TEXTURE0);
    }

    unsafe fn setup_primitive(&mut self) {
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
        let size = (self.vertices.len() * size_of::<Vertex>()) as isize;
        let data = &self.vertices[0] as *const Vertex as *const c_void;
        gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
        let size = (self.indices.len() * size_of::<u32>()) as isize;
        let data = &self.indices[0] as *const u32 as *const c_void;
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        // set the vertex attribute pointers
        // TODO!!: adapt to new Vertex struct
        let size = size_of::<Vertex>() as i32;
        // vertex positions
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, position) as *const c_void);
        // vertex normals
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, normal) as *const c_void);
        // vertex texture coords
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, tex_coord_0) as *const c_void);
        // vertex tangent
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, tangent) as *const c_void);
        // vertex bitangent
        // gl::EnableVertexAttribArray(4);
        // gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, bitangent) as *const c_void);

        gl::BindVertexArray(0);
    }
}
