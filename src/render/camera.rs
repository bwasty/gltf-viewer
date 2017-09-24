use gltf;

pub struct Camera {
    pub znear: f32,
    pub zfar: Option<f32>,

    // perspective camera
    pub fovy: f32,
    pub aspect_ratio: f32,

    // orthographic camera
    pub xmag: Option<f32>,
    pub ymag: Option<f32>,
}

impl Camera {
    pub fn from_gltf(_g_camera: &gltf::Camera) -> Self {
        // TODO!!!
        unimplemented!()
    }
}
