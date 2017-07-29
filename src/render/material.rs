use std::rc::Rc;

use gltf;

use render::math::*;
use render::Scene;
use render::Texture;

pub struct Material {
    pub index: usize, /// glTF index
    pub name: Option<String>,

    pub base_color_factor: Vector4,
    pub base_color_texture: Option<Rc<Texture>>,

    // TODO!!: Material - rest of properties
}

impl Material {
    pub fn from_gltf(g_material: &gltf::material::Material, scene: &mut Scene) -> Material {
        let pbr = g_material.pbr_metallic_roughness()
            .unwrap(); // tmp - see https://github.com/alteous/gltf/issues/48

        let mut texture = None;
        if let Some(base_color_tex_info) = pbr.base_color_texture() {
            let g_texture = &*base_color_tex_info;
            if let Some(tex) = scene.textures.iter().find(|tex| (***tex).index == g_texture.index()) {
                texture = Some(tex.clone());
            }

            if texture.is_none() { // not using else due to borrow-checking madness
                texture = Some(Rc::new(Texture::from_gltf(g_texture)));
                scene.textures.push(texture.clone().unwrap());
            }
        }

        Material {
            index: g_material.index(),
            name: g_material.name().map(|s| s.into()),
            base_color_factor: pbr.base_color_factor().into(),
            // TODO: perhaps RC only the underlying image? no, also opengl id...
            // base_color_texture: Rc::new(Texture)
            base_color_texture: texture
        }
    }
}
