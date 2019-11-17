use gltf;
use gltf::json::texture::MinFilter;

use image::DynamicImage;
use image::DynamicImage::*;
use image::GenericImageView;
use image::FilterType;

use crate::render::texture::Texture;

pub trait TextureHelpers {
    unsafe fn setup_texture(&mut self, g_texture: &gltf::Texture<'_>, dyn_img: &DynamicImage);
}

impl TextureHelpers for Texture {
    unsafe fn setup_texture(&mut self, g_texture: &gltf::Texture<'_>, dyn_img: &DynamicImage) {

    }
}
