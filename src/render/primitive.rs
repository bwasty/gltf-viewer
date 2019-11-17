use std::path::Path;
use std::rc::Rc;

use gltf;
use log::{warn, debug};

use crate::render::math::*;
use crate::render::{Material, Root};
use crate::shader::*;
use crate::importdata::ImportData;

use crate::platform::{PrimitiveHelpers};


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

pub struct Primitive {
    pub bounds: Aabb3,

    pub vao: u32,
    pub vbo: u32,
    pub num_vertices: u32,

    pub ebo: Option<u32>,
    pub num_indices: u32,

    pub mode: u32,

    pub material: Rc<Material>,

    pub pbr_shader: Rc<PbrShader>,

    // TODO!: mode, targets
}

impl Primitive {
    pub fn new(
        bounds: Aabb3,
        vertices: &[Vertex],
        indices: Option<Vec<u32>>,
        mode: u32,
        material: Rc<Material>,
        shader: Rc<PbrShader>,
    ) -> Primitive {
        let num_indices = indices.as_ref().map(|i| i.len()).unwrap_or(0);
        let mut prim = Primitive {
            bounds,
            num_vertices: vertices.len() as u32,
            num_indices: num_indices as u32,
            vao: 0, vbo: 0, ebo: None,
            mode,
            material,
            pbr_shader: shader,
        };

        // now that we have all the required data, set the vertex buffers and its attribute pointers.
        unsafe { prim.setup_primitive(vertices, indices) }
        prim
    }

    pub fn from_gltf(
        g_primitive: &gltf::Primitive<'_>,
        primitive_index: usize,
        mesh_index: usize,
        root: &mut Root,
        imp: &ImportData,
        base_path: &Path) -> Primitive
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
        let mode = g_primitive.mode().as_gl_enum() as u32;

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

        Primitive::new(bounds, &vertices, indices, mode, material, shader)
    }
}
