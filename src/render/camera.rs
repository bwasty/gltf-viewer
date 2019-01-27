use cgmath::{Deg, Rad, perspective};

use gltf;
use gltf::camera::Projection;

use crate::render::math::*;
use crate::controls::{ZOOM};

#[derive(Clone)]
pub struct Camera {
    pub index: usize, // gltf index
    pub name: Option<String>,

    pub projection_matrix: Matrix4,

    pub znear: f32,
    pub zfar: Option<f32>,

    // perspective camera
    // TODO!: setters that update...
    pub fovy: Deg<f32>,
    aspect_ratio: f32,

    // orthographic camera
    pub xmag: Option<f32>,
    pub ymag: Option<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            index: 0,
            name: None,

            znear: 0.01,
            zfar: Some(1000.0),

            fovy: Deg(ZOOM),
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
            index: g_camera.index(),
            name: g_camera.name().map(|n| n.to_owned()),
            projection_matrix: Matrix4::zero(),
            znear: 0.0,
            zfar: None,
            fovy: Deg(0.0),
            aspect_ratio: 1.0,
            xmag: None,
            ymag: None,
        };
        match g_camera.projection() {
            Projection::Perspective(persp) => {
                // TODO!!: ignoring aspect ratio for now as it would require window resizing...
                let _aspect = persp.aspect_ratio();
                camera.fovy = Deg::from(Rad(persp.yfov()));
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

    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn update_projection_matrix(&mut self) {
        if let Some(xmag) = self.xmag {
            // from https://github.com/KhronosGroup/glTF/tree/master/specification/2.0#orthographic-projection
            let r = xmag;
            let t = self.ymag.unwrap();
            let f = self.zfar.unwrap();
            let n = self.znear;
            self.projection_matrix = Matrix4::new(
                1.0/r, 0.0,   0.0,         0.0,   // NOTE: first column!
                0.0,   1.0/t, 0.0,         0.0,   // 2nd
                0.0,   0.0,   2.0/(n-f),   0.0,   // 3rd
                0.0,   0.0,   (f+n)/(n-f), 1.0    // 4th
            );
        } else if let Some(zfar) = self.zfar {
            self.projection_matrix = perspective(
                self.fovy,
                self.aspect_ratio,
                self.znear, zfar)
        } else {
            // from https://github.com/KhronosGroup/glTF/tree/master/specification/2.0#infinite-perspective-projection
            let a = self.aspect_ratio;
            let y = Rad::from(self.fovy).0;
            let n = self.znear;

            self.projection_matrix = Matrix4::new(
                1.0/(a*(0.5*y).tan()), 0.0,               0.0,   0.0, // NOTE: first column!
                0.0,                   1.0/(0.5*y).tan(), 0.0,   0.0,
                0.0,                   0.0,              -1.0,  -1.0,
                0.0,                   0.0,              -2.0*n, 0.0
            );
        }
    }

    pub fn is_perspective(&self) -> bool {
        self.xmag.is_none()
    }

    pub fn description(&self) -> String {
        let type_ = if !self.is_perspective() {
            "ortho"
        } else if self.zfar.is_none() {
            "infinite perspective"
        } else {
            "perspective"
        };

        format!("{} ({:?}, {})", self.index, self.name, type_)
    }
}
