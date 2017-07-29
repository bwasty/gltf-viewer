use std::os::raw::c_void;

use gl;
use gltf;

use image::DynamicImage::*;
use image::GenericImage;

pub struct Texture {
    pub index: usize, // glTF index
    pub name: Option<String>,

    pub id: u32, // OpenGL id

    // TODO notes
    // sampler - merge here?
    // image -> Rc
}

impl Texture {
    pub fn from_gltf(g_texture: &gltf::texture::Texture) -> Texture {

        // TODO!: share images via Rc? detect if occurs?
        let img = g_texture.source();
        let dyn_img = img.data();
        let format = match *dyn_img {
            ImageLuma8(_) => gl::RED,
            ImageLumaA8(_) => gl::RG,
            ImageRgb8(_) => gl::RGB,
            ImageRgba8(_) => gl::RGBA,
        };

        let data = dyn_img.raw_pixels();

        let mut texture_id = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, dyn_img.width() as i32, dyn_img.height() as i32,
                0, format, gl::UNSIGNED_BYTE, &data[0] as *const u8 as *const c_void);

            // TODO!!: get parameters and whether to generate mipmaps from sampler
            // let sampler = g_texture.sampler();
            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        Texture {
            index: g_texture.index(),
            name: g_texture.name().map(|s| s.into()),
            id: texture_id,
        }
    }
}
