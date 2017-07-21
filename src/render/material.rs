use std::os::raw::c_void;
use std::path::Path;

use gl;
use image;
use image::DynamicImage::*;
use image::GenericImage;

use gltf;

use render::math::*;

// TODO!!: Material
pub struct Material {
    pub index: usize, /// glTF index
    pub name: Option<String>,

    pub base_color_factor: Vector4,
    // pub base_color_texture: Rc<Texture>
}

pub fn from_gltf(g_material: &gltf::material::Material) -> Material {
    let pbr = g_material.pbr_metallic_roughness()
        .expect("not yet implemented: material must contain pbr_metallic_roughness");
    Material {
        index: g_material.index(),
        name: g_material.name().map(|s| s.into()),
        base_color_factor: Vector4::from(pbr.base_color_factor()),
        // TODO: perhaps RC only the underlying image? no, also opengl id...
        // base_color_texture: Rc::new(Texture)
    }
}

unsafe fn texture_from_file(path: &str, directory: &str) -> u32 {
    let filename = format!("{}/{}", directory, path);

    let mut texture_id = 0;
    gl::GenTextures(1, &mut texture_id);

    let img = image::open(&Path::new(&filename)).expect("Texture failed to load");
    let format = match img {
        ImageLuma8(_) => gl::RED,
        ImageLumaA8(_) => gl::RG,
        ImageRgb8(_) => gl::RGB,
        ImageRgba8(_) => gl::RGBA,
    };

    let data = img.raw_pixels();

    gl::BindTexture(gl::TEXTURE_2D, texture_id);
    gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, img.width() as i32, img.height() as i32,
        0, format, gl::UNSIGNED_BYTE, &data[0] as *const u8 as *const c_void);
    gl::GenerateMipmap(gl::TEXTURE_2D);

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    texture_id
}
