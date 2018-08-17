use gltf;

/// Helps to simplify the signature of import related functions.
pub struct ImportData {
    pub doc: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

