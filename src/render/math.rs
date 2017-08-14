use cgmath;
pub use cgmath::prelude::*;
pub use cgmath::vec3;

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;

pub type Matrix4 = cgmath::Matrix4<f32>;
pub type Quaternion = cgmath::Quaternion<f32>;

#[derive(Clone)]
pub struct Bounds {
    pub min: Vector3,
    pub max: Vector3,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: Vector3::zero(),
            max: Vector3::zero()
        }
    }
}

impl Bounds {
    pub fn union(&self, other: &Self) -> Bounds {
        Bounds {
            min: vec3(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.x),
                self.min.z.min(other.min.x),
            ),
            max: vec3(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.x),
                self.max.z.max(other.max.x),
            ),
        }
    }
}

use std::convert::From;
use gltf;
impl From<gltf::mesh::Bounds> for Bounds {
    fn from(bounds: gltf::mesh::Bounds) -> Self {
        Bounds {
            min: bounds.min.into(),
            max: bounds.max.into()
        }
    }
}
