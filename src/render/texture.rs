use std::os::raw::c_void;

use gl;
use gltf;
use gltf::json::texture::WrappingMode::*;
use gltf::json::texture::{MinFilter, MagFilter};

use image::DynamicImage::*;
use image::GenericImage;
use image::FilterType;

pub struct Texture {
    pub index: usize, // glTF index
    pub name: Option<String>,

    pub id: u32, // OpenGL id
}

// TODO!!: broken texture rendering: Suzanne (all black; base color factor makes it white)
impl Texture {
    pub fn from_gltf(g_texture: &gltf::texture::Texture) -> Texture {
        let mut texture_id = 0;
        let needs_power_of_two;
        let generate_mip_maps;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            // NOTE: tmp - see https://github.com/alteous/gltf/issues/56
            if let Some(sampler) = g_texture.sampler() {
                let ret = Self::set_sampler_params(sampler);
                needs_power_of_two = ret.0;
                generate_mip_maps = ret.1;
            }
            else {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                needs_power_of_two = true;
                generate_mip_maps = true
            }
        }

        // TODO!: share images via Rc? detect if occurs?
        let img = g_texture.source();
        let dyn_img = img.data();

        let format = match *dyn_img {
            ImageLuma8(_) => gl::RED,
            ImageLumaA8(_) => gl::RG,
            ImageRgb8(_) => gl::RGB,
            ImageRgba8(_) => gl::RGBA,
        };

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

        unsafe {
            gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, width as i32, height as i32,
                0, format, gl::UNSIGNED_BYTE, &data[0] as *const u8 as *const c_void);

            if generate_mip_maps {
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
        }
        Texture {
            index: g_texture.index(),
            name: g_texture.name().map(|s| s.into()),
            id: texture_id,
        }
    }

    // Returns whether image needs to be Power-Of-Two-sized and whether mip maps should be generated
    // TODO: refactor return type into enum?
    unsafe fn set_sampler_params(sampler: gltf::texture::Sampler) -> (bool, bool) {
        // **Mipmapping Implementation Note**: When a sampler's minification filter (`minFilter`)
        // uses mipmapping (`NEAREST_MIPMAP_NEAREST`, `NEAREST_MIPMAP_LINEAR`, `LINEAR_MIPMAP_NEAREST`,
        // or `LINEAR_MIPMAP_LINEAR`), any texture referencing the sampler needs to have mipmaps,
        // e.g., by calling GL's `generateMipmap()` function.
        let mip_maps = match sampler.min_filter() {
            Some(MinFilter::NearestMipmapNearest) |
            Some(MinFilter::LinearMipmapNearest) |
            Some(MinFilter::NearestMipmapLinear) |
            Some(MinFilter::LinearMipmapLinear) => true,
            _ => false
        };

        let wrap_s = match sampler.wrap_s() {
            ClampToEdge => gl::CLAMP_TO_EDGE,
            MirroredRepeat => gl::MIRRORED_REPEAT,
            Repeat => gl::REPEAT,
        };
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_s as i32);
        let wrap_t = match sampler.wrap_t() {
            ClampToEdge => gl::CLAMP_TO_EDGE,
            MirroredRepeat => gl::MIRRORED_REPEAT,
            Repeat => gl::REPEAT,
        };
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_t as i32);

        // **Default Filtering Implementation Note:** When filtering options are defined,
        // runtime must use them. Otherwise, it is free to adapt filtering to performance or quality goals.
        if let Some(min_filter) = sampler.min_filter() {
            let gl_min_filter = match min_filter {
                MinFilter::Nearest => gl::NEAREST,
                MinFilter::Linear => gl::LINEAR,
                MinFilter::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
                MinFilter::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
                MinFilter::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
                MinFilter::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
            };
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl_min_filter as i32);
        }
        else {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
        }
        if let Some(mag_filter) = sampler.mag_filter() {
            let gl_mag_filter = match mag_filter {
                MagFilter::Nearest => gl::NEAREST,
                MagFilter::Linear => gl::LINEAR
            };
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl_mag_filter as i32);
        }
        else {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }

        let needs_power_of_two =
            wrap_s != gl::CLAMP_TO_EDGE ||
            wrap_t != gl::CLAMP_TO_EDGE ||
            mip_maps;
        (needs_power_of_two, mip_maps)
    }
}
