use std::path::Path;
use std::{fs, io};

use base64;
use gltf;
use gltf::image::Source;

use image;
use image::ImageFormat::{JPEG, PNG};

use crate::importdata::ImportData;

use crate::platform::{GltfViewerRenderer,TextureHelpers};

pub struct Texture {
    pub index: usize, // glTF index
    pub name: Option<String>,

    pub id: u32, // OpenGL id
    pub tex_coord: u32, // the tex coord set to use
}

impl Texture {
    pub fn from_gltf(g_texture: &gltf::Texture<'_>, tex_coord: u32, imp: &ImportData, base_path: &Path, renderer: &mut GltfViewerRenderer) -> Texture {
        let buffers = &imp.buffers;
        let mut texture_id = 0;

        // TODO!: share images via Rc? detect if occurs?
        // TODO!!: better I/O abstraction...
        let g_img = g_texture.source();
        let img = match g_img.source() {
            Source::View { view, mime_type } => {
                let parent_buffer_data = &buffers[view.buffer().index()].0;
                let begin = view.offset();
                let end = begin + view.length();
                let data = &parent_buffer_data[begin..end];
                match mime_type {
                    "image/jpeg" => image::load_from_memory_with_format(data, JPEG),
                    "image/png" => image::load_from_memory_with_format(data, PNG),
                    _ => panic!(format!("unsupported image type (image: {}, mime_type: {})",
                        g_img.index(), mime_type)),
                }
            },
            Source::Uri { uri, mime_type } => {
                if uri.starts_with("data:") {
                    let encoded = uri.split(',').nth(1).unwrap();
                    let data = base64::decode(&encoded).unwrap();
                    let mime_type = if let Some(ty) = mime_type {
                        ty
                    } else {
                        uri.split(',')
                            .nth(0).unwrap()
                            .split(':')
                            .nth(1).unwrap()
                            .split(';')
                            .nth(0).unwrap()
                    };

                    match mime_type {
                        "image/jpeg" => image::load_from_memory_with_format(&data, JPEG),
                        "image/png" => image::load_from_memory_with_format(&data, PNG),
                        _ => panic!(format!("unsupported image type (image: {}, mime_type: {})",
                            g_img.index(), mime_type)),
                    }
                }
                else if let Some(mime_type) = mime_type {
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


        let mut texture = Texture {
            index: g_texture.index(),
            name: g_texture.name().map(|s| s.into()),
            id: texture_id,
            tex_coord,
        };
        
        // buffer texture data into graphics platform
        unsafe {
            texture.setup_texture(g_texture, &dyn_img, renderer);
        }
        
        texture
    }
}
