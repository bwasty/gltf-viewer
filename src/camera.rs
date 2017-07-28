use cgmath;
use cgmath::vec3;
use cgmath::prelude::*;

type Point3 = cgmath::Point3<f32>;
type Vector3 = cgmath::Vector3<f32>;
type Matrix4 = cgmath::Matrix4<f32>;

// Defines several possible options for camera movement. Used as abstraction to stay away from window-system specific input methods
#[derive(PartialEq)]
pub enum CameraMovement {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
}
use self::CameraMovement::*;

// Default camera values
const YAW: f32 = -90.0;
const PITCH: f32 = 0.0;
const SPEED: f32 = 2.5;
const SENSITIVTY: f32 = 0.1;
const ZOOM_SENSITIVITY: f32 = 0.1;
const ZOOM: f32 = 45.0;
const MIN_ZOOM: f32 = 1.0;
const MAZ_ZOOM: f32 = 90.0;

pub struct Camera {
    // Camera Attributes
    pub position: Point3,
    pub front: Vector3,
    pub up: Vector3,
    pub right: Vector3,
    pub world_up: Vector3,
    // Euler Angles
    pub yaw: f32,
    pub pitch: f32,
    // Camera options
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    pub zoom: f32,

    // pub moving_up: bool,
    pub moving_left: bool,
    // pub moving_down: bool,
    pub moving_right: bool,
    pub moving_forward: bool,
    pub moving_backward: bool,
}

impl Default for Camera {
    fn default() -> Camera {
        let mut camera = Camera {
            position: Point3::new(0.0, 0.0, 0.0),
            front: vec3(0.0, 0.0, -1.0),
            up: Vector3::zero(), // initialized later
            right: Vector3::zero(), // initialized later
            world_up: Vector3::unit_y(),
            yaw: YAW,
            pitch: PITCH,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVTY,
            zoom: ZOOM,

            // moving_up: false,
            moving_left: false,
            // moving_down: false,
            moving_right: false,
            moving_forward: false,
            moving_backward: false,
        };
        camera.update_camera_vectors();
        camera
    }
}

impl Camera {
    /// Returns the view matrix calculated using Eular Angles and the LookAt Matrix
    pub fn get_view_matrix(&self) -> Matrix4 {
        Matrix4::look_at(self.position, self.position + self.front, self.up)
    }

    pub fn update(&mut self, delta_time: f64) {
        let velocity = self.movement_speed * delta_time as f32;
        if self.moving_forward {
            self.position += self.front * velocity;
        }
        if self.moving_backward {
            self.position += -(self.front * velocity);
        }
        if self.moving_left {
            self.position += -(self.right * velocity);
        }
        if self.moving_right {
            self.position += self.right * velocity;
        }
    }

    pub fn process_keyboard(&mut self, direction: CameraMovement, pressed: bool) {
        match direction {
            FORWARD => self.moving_forward = pressed,
            BACKWARD => self.moving_backward= pressed,
            LEFT => self.moving_left = pressed,
            RIGHT => self.moving_right = pressed,
        }
    }

    /// Processes input received from a mouse input system. Expects the offset value in both the x and y direction.
    pub fn process_mouse_movement(&mut self, mut xoffset: f32, mut yoffset: f32, constrain_pitch: bool) {
        xoffset *= self.mouse_sensitivity;
        yoffset *= self.mouse_sensitivity;

        self.yaw += xoffset;
        self.pitch += yoffset;

        // Make sure that when pitch is out of bounds, screen doesn't get flipped
        if constrain_pitch {
            if self.pitch > 89.0 {
                self.pitch = 89.0;
            }
            if self.pitch < -89.0 {
                self.pitch = -89.0;
            }
        }

        // Update front, Right and Up Vectors using the updated Eular angles
        self.update_camera_vectors();
    }

    // Processes input received from a mouse scroll-wheel event. Only requires input on the vertical wheel-axis
    pub fn process_mouse_scroll(&mut self, mut yoffset: f32) {
        yoffset *= ZOOM_SENSITIVITY;
        if self.zoom >= MIN_ZOOM && self.zoom <= MAZ_ZOOM {
            self.zoom -= yoffset;
        }
        if self.zoom <= MIN_ZOOM {
            self.zoom = MIN_ZOOM;
        }
        if self.zoom >= MAZ_ZOOM {
            self.zoom = MAZ_ZOOM;
        }
    }

    /// Calculates the front vector from the Camera's (updated) Eular Angles
    fn update_camera_vectors(&mut self) {
        // Calculate the new front vector
        let front = Vector3 {
            x: self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            y: self.pitch.to_radians().sin(),
            z: self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        };
        self.front = front.normalize();
        // Also re-calculate the Right and Up vector
        self.right = self.front.cross(self.world_up).normalize(); // Normalize the vectors, because their length gets closer to 0 the more you look up or down which results in slower movement.
        self.up = self.right.cross(self.front).normalize();
    }
}
