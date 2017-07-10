// use std::rc::Rc;

use gltf;

use shader::Shader;
use render::Primitive;

pub struct Mesh {
    pub primitives: Vec<Primitive>,
    // TODO
    // pub weights: Vec<Rc<?>>
    pub name: Option<String>
}

impl Mesh {
    pub fn from_gltf(g_mesh: gltf::mesh::Mesh) -> Mesh {
        Mesh {
            primitives: g_mesh.primitives().map(Primitive::from_gltf).collect(),
            name: g_mesh.name().map(|s| s.into()),
        }
    }

    pub fn draw(&self, shader: &Shader) {
        for primitive in &self.primitives {
            unsafe { primitive.draw(shader) }
        }
    }
}
