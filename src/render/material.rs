use std::rc::Rc;
use std::path::Path;

use gltf;
use gltf_importer;

use render::math::*;
use render::{ Scene, Texture };
use shader::*;

pub struct Material {
    pub index: Option<usize>, /// glTF index
    pub name: Option<String>,

    // pbr_metallic_roughness properties
    pub base_color_factor: Vector4,
    pub base_color_texture: Option<Rc<Texture>>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Option<Rc<Texture>>,

    pub normal_texture: Option<Rc<Texture>>,
    pub normal_scale: Option<f32>,

    pub occlusion_texture: Option<Rc<Texture>>,
    pub occlusion_strength: f32,
    pub emissive_factor: Vector3,
    pub emissive_texture: Option<Rc<Texture>>,

    pub alpha_cutoff: f32,
    pub alpha_mode: gltf::material::AlphaMode,

    pub double_sided: bool,

}

impl Material {
    pub fn from_gltf(
        g_material: &gltf::material::Material,
        scene: &mut Scene,
        buffers: &gltf_importer::Buffers, base_path: &Path
    ) -> Material {
        let pbr = g_material.pbr_metallic_roughness();

        let mut material = Material {
            index: g_material.index(),
            name: g_material.name().map(|s| s.into()),
            base_color_factor: pbr.base_color_factor().into(),
            // TODO: perhaps RC only the underlying image? no, also opengl id...
            base_color_texture: None,
            metallic_factor: pbr.metallic_factor().into(),
            roughness_factor: pbr.roughness_factor().into(),
            metallic_roughness_texture: None,

            normal_texture: None,
            normal_scale: None,

            occlusion_texture: None,
            occlusion_strength: 0.0,

            emissive_factor: g_material.emissive_factor().into(),
            emissive_texture: None,

            alpha_cutoff: g_material.alpha_cutoff(),
            alpha_mode: g_material.alpha_mode(),

            double_sided: g_material.double_sided(),
        };

        if let Some(color_info) = pbr.base_color_texture() {
            material.base_color_texture = Some(
                load_texture(&color_info.texture(), color_info.tex_coord(), scene, buffers, base_path));
        }
        if let Some(mr_info) = pbr.metallic_roughness_texture() {
            material.metallic_roughness_texture = Some(
                load_texture(&mr_info.texture(), mr_info.tex_coord(), scene, buffers, base_path));
        }
        if let Some(normal_texture) = g_material.normal_texture() {
            material.normal_texture = Some(
                load_texture(&normal_texture.texture(), normal_texture.tex_coord(), scene, buffers, base_path));
            material.normal_scale = Some(normal_texture.scale());
        }
        if let Some(occ_texture) = g_material.occlusion_texture() {
            material.occlusion_texture = Some(
                load_texture(&occ_texture.texture(), occ_texture.tex_coord(), scene, buffers, base_path));
            material.occlusion_strength = occ_texture.strength();
        }
        if let Some(em_info) = g_material.emissive_texture() {
            material.emissive_texture = Some(
                load_texture(&em_info.texture(), em_info.tex_coord(), scene, buffers, base_path));
        }

        material
    }

    pub fn shader_flags(&self) -> ShaderFlags {
        let mut flags = ShaderFlags::empty();
        if self.base_color_texture.is_some() {
            flags |= HAS_BASECOLORMAP;
        }
        if self.normal_texture.is_some() {
            flags |= HAS_NORMALMAP;
        }
        if self.emissive_texture.is_some() {
            flags |= HAS_EMISSIVEMAP;
        }
        if self.metallic_roughness_texture.is_some() {
            flags |= HAS_METALROUGHNESSMAP;
        }
        if self.occlusion_texture.is_some() {
            flags |= HAS_OCCLUSIONMAP;
        }
        flags
    }

}

fn load_texture(
    g_texture: &gltf::texture::Texture,
    tex_coord: u32,
    scene: &mut Scene,
    buffers: &gltf_importer::Buffers,
    base_path: &Path) -> Rc<Texture>
{
    // TODO!: handle tex coord set in shaders
    assert_eq!(tex_coord, 0, "not yet implemented: tex coord set must be 0 (Material::from_gltf)");

    if let Some(tex) = scene.textures.iter().find(|tex| (***tex).index == g_texture.index()) {
        return tex.clone()
    }

    let texture = Rc::new(Texture::from_gltf(g_texture, tex_coord, buffers, base_path));
    scene.textures.push(texture.clone());
    texture
}
