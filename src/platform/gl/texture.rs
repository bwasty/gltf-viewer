use std::os::raw::c_void;

use gl;
use gltf;
use gltf::json::texture::MinFilter;

use image::DynamicImage;
use image::DynamicImage::*;
use image::GenericImageView;
use image::FilterType;

use crate::render::texture::Texture;
use crate::platform::{GltfViewerRenderer};

pub trait TextureHelpers {
    unsafe fn setup_texture(&mut self, g_texture: &gltf::Texture<'_>, dyn_img: &DynamicImage, _renderer: &GltfViewerRenderer);
}

impl TextureHelpers for Texture {
    unsafe fn setup_texture(&mut self, g_texture: &gltf::Texture<'_>, dyn_img: &DynamicImage, _renderer: &GltfViewerRenderer) {
        // get texture id from gl
        gl::GenTextures(1, &mut self.id);
        gl::BindTexture(gl::TEXTURE_2D, self.id);
        
        let (needs_power_of_two, generate_mip_maps) =
            set_sampler_params(&g_texture.sampler());
        
        // **Non-Power-Of-Two Texture Implementation Note**: glTF does not guarantee that a texture's
        // dimensions are a power-of-two.  At runtime, if a texture's width or height is not a
        // power-of-two, the texture needs to be resized so its dimensions are powers-of-two if the
        // `sampler` the texture references
        // * Has a wrapping mode (either `wrapS` or `wrapT`) equal to `REPEAT` or `MIRRORED_REPEAT`, or
        // * Has a minification filter (`minFilter`) that uses mipmapping (`NEAREST_MIPMAP_NEAREST`, \\
        //   `NEAREST_MIPMAP_LINEAR`, `LINEAR_MIPMAP_NEAREST`, or `LINEAR_MIPMAP_LINEAR`).
        let (width, height) = dyn_img.dimensions();
        let (data, width, height) =
            if needs_power_of_two && (!width.is_power_of_two() || !height.is_power_of_two()) {
                let nwidth = width.next_power_of_two();
                let nheight = height.next_power_of_two();
                let resized = dyn_img.resize(nwidth, nheight, FilterType::Lanczos3);
                (resized.raw_pixels(), resized.width(), resized.height())
            }
            else {
                (dyn_img.raw_pixels(), dyn_img.width(), dyn_img.height())
            };
        
        let format = match dyn_img {
            ImageLuma8(_) => gl::RED,
            ImageLumaA8(_) => gl::RG,
            ImageRgb8(_) => gl::RGB,
            ImageRgba8(_) => gl::RGBA,
            ImageBgr8(_) => gl::BGR,
            ImageBgra8(_) => gl::BGRA,
        };

        gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, width as i32, height as i32,
            0, format, gl::UNSIGNED_BYTE, &data[0] as *const u8 as *const c_void);

        if generate_mip_maps {
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
    }
}

// Returns whether image needs to be Power-Of-Two-sized and whether mip maps should be generated
// TODO: refactor return type into enum?
unsafe fn set_sampler_params(sampler: &gltf::texture::Sampler<'_>) -> (bool, bool) {
    // **Mipmapping Implementation Note**: When a sampler's minification filter (`minFilter`)
    // uses mipmapping (`NEAREST_MIPMAP_NEAREST`, `NEAREST_MIPMAP_LINEAR`, `LINEAR_MIPMAP_NEAREST`,
    // or `LINEAR_MIPMAP_LINEAR`), any texture referencing the sampler needs to have mipmaps,
    // e.g., by calling GL's `generateMipmap()` function.
    let mip_maps = match sampler.min_filter() {
        Some(MinFilter::NearestMipmapNearest) |
        Some(MinFilter::LinearMipmapNearest) |
        Some(MinFilter::NearestMipmapLinear) |
        Some(MinFilter::LinearMipmapLinear) |
        None => true, // see below
        _ => false
    };

    // **Default Filtering Implementation Note:** When filtering options are defined,
    // runtime must use them. Otherwise, it is free to adapt filtering to performance or quality goals.
    if let Some(min_filter) = sampler.min_filter() {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter.as_gl_enum() as i32);
    }
    else {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
    }
    if let Some(mag_filter) = sampler.mag_filter() {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter.as_gl_enum() as i32);
    }
    else {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    let wrap_s = sampler.wrap_s().as_gl_enum();
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_s as i32);
    let wrap_t = sampler.wrap_t().as_gl_enum();
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_t as i32);

    let needs_power_of_two =
        wrap_s != gl::CLAMP_TO_EDGE ||
        wrap_t != gl::CLAMP_TO_EDGE ||
        mip_maps;
    (needs_power_of_two, mip_maps)
}
