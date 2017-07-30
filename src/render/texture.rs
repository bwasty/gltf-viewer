use std::os::raw::c_void;

use gl;
use gltf;
use gltf::json::texture::WrappingMode::*;
use gltf::json::texture::{MinFilter, MagFilter};

use image::DynamicImage::*;
use image::GenericImage;
use image::FilterType;

use utils::{is_power_of_two, next_power_of_two};

pub struct Texture {
    pub index: usize, // glTF index
    pub name: Option<String>,

    pub id: u32, // OpenGL id
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

        // TODO: make nicer (borrow checker problems...)
        // TODO!!: add spec conditions to the check:
        // - Has a wrapping mode (either wrapS or wrapT) equal to REPEAT or MIRRORED_REPEAT, or
        // - Has a minification filter (minFilter) that uses mipmapping (NEAREST_MIPMAP_NEAREST, NEAREST_MIPMAP_LINEAR, LINEAR_MIPMAP_NEAREST, or LINEAR_MIPMAP_LINEAR).
        let (data, width, height) =
            if !is_power_of_two(dyn_img.width()) || !is_power_of_two(dyn_img.height()) {
                let nwidth = next_power_of_two(dyn_img.width());
                let nheight = next_power_of_two(dyn_img.height());
                let resized = dyn_img.resize(nwidth, nheight, FilterType::Lanczos3);
                (resized.raw_pixels(), resized.width(), resized.height())
            }
            else {
                (dyn_img.raw_pixels(), dyn_img.width(), dyn_img.height())
            };

        let mut texture_id = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, width as i32, height as i32,
                0, format, gl::UNSIGNED_BYTE, &data[0] as *const u8 as *const c_void);

            // NOTE: tmp - see https://github.com/alteous/gltf/issues/56
            if let Some(sampler) = g_texture.sampler() {
                Self::set_sampler_params(sampler);
            }
            else {
                gl::GenerateMipmap(gl::TEXTURE_2D);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            }
        }
        Texture {
            index: g_texture.index(),
            name: g_texture.name().map(|s| s.into()),
            id: texture_id,
        }
    }

    unsafe fn set_sampler_params(sampler: gltf::texture::Sampler) {
        let mip_maps = match sampler.min_filter() {
            Some(MinFilter::NearestMipmapNearest) |
            Some(MinFilter::LinearMipmapNearest) |
            Some(MinFilter::NearestMipmapLinear) |
            Some(MinFilter::LinearMipmapLinear) => true,
            _ => false
        };
        if mip_maps {
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, match sampler.wrap_s() {
            ClampToEdge => gl::CLAMP_TO_EDGE,
            MirroredRepeat => gl::MIRRORED_REPEAT,
            Repeat => gl::REPEAT,
        } as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, match sampler.wrap_t() {
            ClampToEdge => gl::CLAMP_TO_EDGE,
            MirroredRepeat => gl::MIRRORED_REPEAT,
            Repeat => gl::REPEAT,
        } as i32);

        // TODO!!: choose good default filtering mode...
        // SPEC: Default Filtering Implementation Note: When filtering options are defined, runtime must use them.
        // Otherwise, it is free to adapt filtering to performance or quality goals.
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
        if let Some(mag_filter) = sampler.mag_filter() {
            let gl_mag_filter = match mag_filter {
                MagFilter::Nearest => gl::NEAREST,
                MagFilter::Linear => gl::LINEAR
            };
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl_mag_filter as i32);
        }
    }
}
