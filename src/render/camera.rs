use cgmath::{Deg, perspective};

use gltf;
use gltf::camera::Projection;

use render::math::*;
use ::controls::{ZOOM};

#[derive(Clone)]
pub struct Camera {
    pub projection_matrix: Matrix4,

    pub znear: f32,
    pub zfar: Option<f32>,

    // perspective camera
    // TODO!: setters that update...
    pub fovy: f32,
    aspect_ratio: f32,

    // orthographic camera
    pub xmag: Option<f32>,
    pub ymag: Option<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            znear: 0.01,
            zfar: Some(1000.0),

            fovy: ZOOM,
            aspect_ratio: 1.0,

            xmag: None,
            ymag: None,

            projection_matrix: Matrix4::zero(),
        }
    }
}

impl Camera {
    pub fn from_gltf(g_camera: &gltf::Camera) -> Self {
        let mut camera = Camera {
            projection_matrix: Matrix4::zero(),
            znear: 0.0,
            zfar: None,
            fovy: 0.0,
            aspect_ratio: 1.0,
            xmag: None,
            ymag: None,
        };
        match g_camera.projection() {
            Projection::Perspective(persp) => {
                // TODO!: ignoring aspect ratio for now as it would require window resizing...
                let _aspect = persp.aspect_ratio();
                camera.fovy = persp.yfov();
                camera.znear = persp.znear();
                camera.zfar = persp.zfar();
            },
            Projection::Orthographic(ortho) => {
                camera.xmag = Some(ortho.xmag());
                camera.ymag = Some(ortho.ymag());
                camera.znear = ortho.znear();
                camera.zfar = Some(ortho.zfar());
            }
        }
        camera.update_projection_matrix();
        camera
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.update_projection_matrix();
    }

    pub fn update_projection_matrix(&mut self) {
        if let Some(_xmag) = self.xmag {
            unimplemented!("orthographic camera") // TODO!!!: ortho camera
        } else if let Some(zfar) = self.zfar {
            self.projection_matrix = perspective(
                Deg(self.fovy),
                self.aspect_ratio,
                self.znear, zfar)
        } else {
            // TODO!!: inifinite perspective (missing sample models)
            unimplemented!("infinite perspective")
        }
    }
}
