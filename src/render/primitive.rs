use std::path::Path;
use std::rc::Rc;

use gltf;

use crate::render::math::*;
use crate::render::{Material, Root};
use crate::shader::*;
use crate::importdata::ImportData;
use crate::platform::{GltfViewerRenderer,PrimitiveHelpers};

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

    pub material: Option<Rc<Material>>,

    pub pbr_shader: Option<Rc<PbrShader>>,

    // TODO!: mode, targets
}

impl Primitive {
    pub fn new(
        bounds: Aabb3,
        mode: u32,
    ) -> Primitive {
        Primitive {
            bounds,
            num_vertices: 0,
            num_indices: 0,
            vao: 0, vbo: 0, ebo: None,
            mode,
            material: None,
            pbr_shader: None,
        }
    }

    pub fn from_gltf(
        g_primitive: &gltf::Primitive<'_>,
        _primitive_index: usize,
        _mesh_index: usize,
        root: &mut Root,
        imp: &ImportData,
        base_path: &Path,
        renderer: &mut GltfViewerRenderer,
    ) -> Primitive
    {
        // bounding box
        let bounds = g_primitive.bounding_box();
        let bounds = Aabb3 {
            min: bounds.min.into(),
            max: bounds.max.into()
        };

        // TODO: spec:
        // Implementation note: When the 'mode' property is set to a non-triangular type
        //(such as POINTS or LINES) some additional considerations must be taken while
        //considering the proper rendering technique:
        //   For LINES with NORMAL and TANGENT properties can render with standard lighting including normal maps.
        //   For all POINTS or LINES with no TANGENT property, render with standard lighting but ignore any normal maps on the material.
        //   For POINTS or LINES with no NORMAL property, don't calculate lighting and instead output the COLOR value for each pixel drawn.
        let mode = g_primitive.mode().as_gl_enum() as u32;

        let mut prim = Primitive::new(bounds, mode);

        unsafe {
            prim.setup_primitive(g_primitive, imp, root, base_path, renderer)
        };

        prim
    }
}
