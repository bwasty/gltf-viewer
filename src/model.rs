use std::os::raw::c_void;
use std::path::Path;

// use cgmath::{vec2, vec3};
use gl;
use image;
use image::DynamicImage::*;
use image::GenericImage;

use mesh::{ Mesh, Texture, /*Vertex*/ };
use shader::Shader;

#[derive(Default)]
pub struct Model {
    /*  Model Data */
    pub meshes: Vec<Mesh>,
    textures_loaded: Vec<Texture>,   // stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    directory: String,
}

impl Model {
    /// constructor, expects a filepath to a 3D model.
    pub fn new(_path: &str) -> Model {
        let model = Model::default();
        // model.load_model(path);
        model
    }

    pub fn draw(&self, shader: &Shader) {
        for mesh in &self.meshes {
            unsafe { mesh.draw(shader); }
        }
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
