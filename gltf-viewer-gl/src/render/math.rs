use std::f32;

use cgmath;
pub use cgmath::prelude::*;
pub use cgmath::{vec3, vec4};

use num_traits::clamp;

use collision;

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;

pub type Point3 = cgmath::Point3<f32>;

pub type Matrix4 = cgmath::Matrix4<f32>;
pub type Quaternion = cgmath::Quaternion<f32>;

pub type Aabb3 = collision::Aabb3<f32>;

// A point's spherical coordinates, inspired by ThreeJS version
pub struct Spherical {
    pub radius: f32,
    pub phi: f32,
    pub theta: f32,
}

impl Default for Spherical {
    fn default() -> Self {
        Spherical { radius: 1.0, phi: 0.0, theta: 0.0 }
    }
}

impl Spherical {
    pub fn from_vec3(vec3: Vector3) -> Self {
        let radius = vec3.magnitude();
        let (theta, phi) = if radius == 0.0 {
            (0.0, 0.0)
        } else {
            (
                vec3.x.atan2(vec3.z), // equator angle around y-up axis
                clamp(vec3.y / radius, -1.0, 1.0).acos() // polar angle
            )
        };
        Self {
            radius,
            theta,
            phi
        }
    }

    pub fn to_vec3(&self) -> Vector3 {
        let sin_phi_radius = self.phi.sin() * self.radius;
        let x = sin_phi_radius * self.theta.sin();
        let y = self.phi.cos() * self.radius;
        let z = sin_phi_radius * self.theta.cos();
        vec3(x, y, z)
    }
}

use std::num::ParseFloatError;
pub fn parse_vec3(s: &str) -> Result<Vector3, ParseFloatError> {
    let coords: Vec<&str> = s.split(',').collect();
    assert!(coords.len() == 3, "Failed to parse Vector3 ({})", s);
    let x = coords[0].parse::<f32>()?;
    let y = coords[1].parse::<f32>()?;
    let z = coords[2].parse::<f32>()?;

    Ok(vec3(x, y, z))
}
