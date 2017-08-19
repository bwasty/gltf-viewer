use std::os::raw::c_void;
use std::path::Path;
use std::{fs, io};

use gl;
use gltf;
use gltf::json::texture::MinFilter;
use gltf::image::Data;
use gltf_importer;

use image;
use image::ImageFormat::{JPEG, PNG};
use image::DynamicImage::*;
use image::GenericImage;
use image::FilterType;

pub struct Texture {
    pub index: usize, // glTF index
    pub name: Option<String>,

    pub id: u32, // OpenGL id
    pub tex_coord: u32, // the tex coord set to use
}

impl Texture {
    pub fn from_gltf(g_texture: &gltf::Texture, tex_coord: u32, buffers: &gltf_importer::Buffers, base_path: &Path) -> Texture {
        let mut texture_id = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
        }
        let (needs_power_of_two, generate_mip_maps) =
            unsafe { Self::set_sampler_params(g_texture.sampler()) };

        // TODO!: share images via Rc? detect if occurs?
        // TODO!!: better I/O abstraction...
        let g_img = g_texture.source();
        let img = match g_img.data() {
            Data::View { view, mime_type } => {
                let data = buffers.view(&view).expect("Failed to get buffer view for image");
                match mime_type {
                    "image/jpeg" => image::load_from_memory_with_format(data, JPEG),
                    "image/png" => image::load_from_memory_with_format(data, PNG),
                    _ => panic!(format!("unsupported image type (image: {}, mime_type: {})",
                        g_img.index(), mime_type)),
                }
            },
            Data::Uri { uri, mime_type } => {
                if let Some(mime_type) = mime_type {
                    let path = base_path.parent().unwrap_or_else(|| Path::new("./")).join(uri);
                    let file = fs::File::open(path).unwrap();
                    let reader = io::BufReader::new(file);
                    match mime_type {
                        "image/jpeg" => image::load(reader, JPEG),
                        "image/png" => image::load(reader, PNG),
                        _ => panic!(format!("unsupported image type (image: {}, mime_type: {})",
                            g_img.index(), mime_type)),
                    }
                }
                else {
                    let path = base_path.parent().unwrap_or_else(||Path::new("./")).join(uri);
                    image::open(path)
                }
            }
        };

        // TODO: handle I/O problems
        let dyn_img = img.expect("Image loading failed.");

        let format = match dyn_img {
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
            tex_coord: tex_coord,
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
            Some(MinFilter::LinearMipmapLinear) |
            None => true, // see below
            _ => false
        };

        // **Default Filtering Implementation Note:** When filtering options are defined,
        // runtime must use them. Otherwise, it is free to adapt filtering to performance or quality goals.
        if let Some(min_filter) = sampler.min_filter() {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter.as_gl_enum());
        }
        else {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
        }
        if let Some(mag_filter) = sampler.mag_filter() {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter.as_gl_enum());
        }
        else {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }

        let wrap_s = sampler.wrap_s().as_gl_enum();
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_s);
        let wrap_t = sampler.wrap_t().as_gl_enum();
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_t);

        let needs_power_of_two =
            wrap_s != gl::CLAMP_TO_EDGE as i32 ||
            wrap_t != gl::CLAMP_TO_EDGE as i32 ||
            mip_maps;
        (needs_power_of_two, mip_maps)
    }
}
