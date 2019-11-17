use std::rc::Rc;

use gltf;
use gltf::json::texture::MinFilter;

use image::DynamicImage;
use image::DynamicImage::*;
use image::GenericImageView;
use image::FilterType;

use web_sys::{WebGlTexture};
use web_sys::WebGl2RenderingContext as GL;

use crate::{debug};
use crate::render::texture::Texture;
use crate::platform::{GltfViewerRenderer};

pub trait TextureHelpers {
    unsafe fn setup_texture(&mut self, g_texture: &gltf::Texture<'_>, dyn_img: &DynamicImage, renderer: &mut GltfViewerRenderer);
}

impl TextureHelpers for Texture {
    unsafe fn setup_texture(&mut self, g_texture: &gltf::Texture<'_>, dyn_img: &DynamicImage, renderer: &mut GltfViewerRenderer) {
        let gl = renderer.gl.as_ref();

        // create texture reference and store on renderer object
        let texture = Rc::new(gl.create_texture().expect("failed to create buffer for texture"));
        self.id = renderer.textures.len() as u32;
        renderer.textures.push(texture);
        let texture_id = renderer.textures[self.id as usize].as_ref();

        // bind 
        gl.bind_texture(GL::TEXTURE_2D, Some(texture_id));

        // set texture sampler params
        let (needs_power_of_two, generate_mip_maps) = set_sampler_params(&g_texture.sampler(), renderer);

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

        // image format. webgl only supports these two
        let format = match dyn_img {
            ImageRgb8(_) => GL::RGB,
            ImageRgba8(_) => GL::RGBA,
            _ => panic!("unsupported color format"),
        };

        // write image data to buffer
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            GL::TEXTURE_2D,
            0,
            format as i32,
            width as i32,
            height as i32,
            0,
            format,
            GL::UNSIGNED_BYTE,
            Some(&data)
        );
        debug!("texture bound {}", self.id);

        if generate_mip_maps {
            gl.generate_mipmap(GL::TEXTURE_2D);
        }
    }
}

// Returns whether image needs to be Power-Of-Two-sized and whether mip maps should be generated
// TODO: refactor return type into enum?
unsafe fn set_sampler_params(sampler: &gltf::texture::Sampler<'_>, renderer: &GltfViewerRenderer) -> (bool, bool) {
    let gl = renderer.gl.as_ref();

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
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, min_filter.as_gl_enum() as i32);
    }
    else {
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR_MIPMAP_LINEAR as i32);
    }
    if let Some(mag_filter) = sampler.mag_filter() {
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, mag_filter.as_gl_enum() as i32);
    }

    else {
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
    }

    let wrap_s = sampler.wrap_s().as_gl_enum();
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, wrap_s as i32);
    let wrap_t = sampler.wrap_t().as_gl_enum();
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, wrap_t as i32);

    let needs_power_of_two =
        wrap_s != GL::CLAMP_TO_EDGE ||
        wrap_t != GL::CLAMP_TO_EDGE ||
        mip_maps;
    (needs_power_of_two, mip_maps)
}
