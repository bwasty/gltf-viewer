use std::f32::consts::PI;

use cgmath::{vec3, Deg};
use cgmath::prelude::*;

use log::{warn, trace};
use num_traits::clamp;

// type Point3 = cgmath::Point3<f32>;
// type Vector3 = cgmath::Vector3<f32>;
// type Matrix4 = cgmath::Matrix4<f32>;

use crate::render::Camera;
use crate::render::math::*;

use winit::dpi::PhysicalPosition;
use winit::dpi::PhysicalSize;

// Defines several possible options for camera movement. Used as abstraction to stay away from window-system specific input methods
#[derive(PartialEq, Clone, Copy)]
pub enum CameraMovement {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
}
use self::CameraMovement::*;

#[derive(Debug)]
pub struct CameraParams {
    pub position: Vector3,
    pub view_matrix: Matrix4,
    pub projection_matrix: Matrix4,
}

// Default camera values
const SPEED: f32 = 2.5;
const ZOOM_SENSITIVITY: f32 = 0.1;
pub const ZOOM: f32 = 45.0;
const MIN_ZOOM: f32 = 1.0;
const MAZ_ZOOM: f32 = 170.0;

#[derive(Clone)]
pub enum NavState {
    None,
    Rotating,
    Panning,
}

/// Inspirted by `ThreeJS` `OrbitControls`
pub struct OrbitControls {
    pub camera: Camera,

    pub position: Point3,

    // "target" sets the location of focus, where the object orbits around
    pub target: Point3,

    pub state: NavState,

    // current position in spherical coordinates
    spherical: Spherical,
    spherical_delta: Spherical,

    scale: f32,
    pan_offset: Vector3,

    rotate_start: Option<Vector2>,
    rotate_end: Vector2,

    pan_start: Option<Vector2>,
    pan_end: Vector2,

    // for keyboard nav
    // pub moving_up: bool,
    pub moving_left: bool,
    // pub moving_down: bool,
    pub moving_right: bool,
    pub moving_forward: bool,
    pub moving_backward: bool,

    pub screen_size: PhysicalSize<f64>,
}

impl OrbitControls {
    pub fn new(position: Point3, screen_size: PhysicalSize<f64>) -> Self {
        OrbitControls {
            camera: Camera::default(),

            position,
            target: Point3::new(0.0, 0.0, 0.0),

            state: NavState::None,

            // current position in spherical coordinates
            spherical: Spherical::default(),
            spherical_delta: Spherical::default(),

            scale: 1.0, // TODO!: not really used
            pan_offset: Vector3::zero(),

            rotate_start: None,
            rotate_end: Vector2::zero(),

            pan_start: None,
            pan_end: Vector2::zero(),

            // moving_up: false,
            moving_left: false,
            // moving_down: false,
            moving_right: false,
            moving_forward: false,
            moving_backward: false,

            screen_size,
        }
    }

    // NOTE: could be cached
    pub fn camera_params(&self) -> CameraParams {
        CameraParams {
            position: self.position.to_vec(),
            view_matrix: self.view_matrix(),
            projection_matrix: self.camera.projection_matrix,
        }
    }

    fn view_matrix(&self) -> Matrix4 {
        Matrix4::look_at(self.position, self.target, vec3(0.0, 1.0, 0.0))
    }

    pub fn handle_mouse_move(&mut self, pos: PhysicalPosition<f64>) {
        match self.state {
            NavState::Rotating => self.handle_mouse_move_rotate(pos),
            NavState::Panning => self.handle_mouse_move_pan(pos),
            NavState::None => ()
        }
    }

    fn handle_mouse_move_rotate(&mut self, pos: PhysicalPosition<f64>) {
        self.rotate_end.x = pos.x as f32;
        self.rotate_end.y = pos.y as f32;
        let rotate_delta = if let Some(rotate_start) = self.rotate_start {
            self.rotate_end - rotate_start
        } else {
            Vector2::zero()
        };

        // rotating across whole screen goes 360 degrees around
        let rotate_speed = 1.0; // TODO: const/param/remove?
        let angle = 2.0 * PI * rotate_delta.x / self.screen_size.width as f32 * rotate_speed;
        self.rotate_left(angle);

        // rotating up and down along whole screen attempts to go 360, but limited to 180
        let angle = 2.0 * PI * rotate_delta.y / self.screen_size.height as f32 * rotate_speed;
        self.rotate_up(angle);

        self.rotate_start = Some(self.rotate_end);

        self.update();
    }

    pub fn handle_mouse_up(&mut self) {
        self.rotate_start = None;
        self.pan_start = None;
    }

    fn rotate_left(&mut self, angle: f32) {
        self.spherical_delta.theta -= angle;
    }

    pub fn rotate_object(&mut self, angle: f32) {
        self.rotate_left(angle);
        self.update();
    }
    fn rotate_up(&mut self, angle: f32) {
        self.spherical_delta.phi -= angle;
    }

    fn handle_mouse_move_pan(&mut self, pos: PhysicalPosition<f64>) {
        self.pan_end.x = pos.x as f32;
        self.pan_end.y = pos.y as f32;

        let pan_delta = if let Some(pan_start) = self.pan_start {
            self.pan_end - pan_start
        } else {
            Vector2::zero()
        };

        self.pan(pan_delta);

        self.pan_start = Some(self.pan_end);

        self.update();
    }

    fn pan(&mut self, delta: Vector2) {
        if self.camera.is_perspective() {
            let offset = self.position - self.target;
            let mut target_distance = offset.magnitude();

            // half of the fov is center to top of screen
            target_distance *= (self.camera.fovy / 2.0).tan() * PI / 180.0;

            // we actually don't use screen_width, since perspective camera is fixed to screen height
            let distance = 50.0 * delta.x * target_distance / self.screen_size.height as f32;
            self.pan_left(-distance);
            let distance = 50.0 * delta.y * target_distance / self.screen_size.height as f32;
            self.pan_up(-distance);
        } else {
            // TODO!: orthographic camera pan
            warn!("unimplemented: orthographic camera pan")
        }
    }

    pub fn pan_left(&mut self, distance: f32) {
        self.pan_offset.x -= distance
    }

    pub fn pan_up(&mut self, distance: f32) {
        self.pan_offset.y -= distance
    }

    pub fn pan_both(&mut self, delta: PhysicalPosition<f32>) {
        self.pan_offset.x += delta.x;
        self.pan_offset.y += delta.y;
        self.update();
    }

    // Processes input received from a mouse scroll-wheel event. Only requires input on the vertical wheel-axis
    pub fn process_mouse_scroll(&mut self, mut yoffset: f32) {
        yoffset *= ZOOM_SENSITIVITY;
        if self.camera.fovy.0 >= MIN_ZOOM && self.camera.fovy.0 <= MAZ_ZOOM {
            self.camera.fovy.0 -= yoffset;
        }
        if self.camera.fovy.0 <= MIN_ZOOM {
            self.camera.fovy.0 = MIN_ZOOM;
        }
        if self.camera.fovy.0 >= MAZ_ZOOM {
            self.camera.fovy.0 = MAZ_ZOOM;
        }
        self.camera.update_projection_matrix();
    }

    /// Update camera after processing mouse events
    fn update(&mut self) {
        let mut offset = self.position - self.target;

        // NOTE: skipping rotate offset to "y-axis-is-up" space

        // angle from z-axis around y-axis
        self.spherical = Spherical::from_vec3(offset);

        self.spherical.theta += self.spherical_delta.theta;
        self.spherical.phi += self.spherical_delta.phi;

        // NOTE!: left out theta restrictions / make_safe for now

        // restrict phi to be between desired limits
        let epsilon = 0.0001;
        self.spherical.phi = clamp(self.spherical.phi, epsilon, PI - epsilon);

        self.spherical.radius *= self.scale;

        // TODO?: restrict radius to be between desired limits?

        // move target to panned location
        // NOTE: quite different from original
        // NOTE: skipped from original: rotate offset back to "camera-up-vector-is-up" space
        let pan_speed = 2.0; // TODO!!: test on non-retina display
        self.pan_offset *= pan_speed;
        let right = offset.cross(Vector3::unit_y()).normalize();
        let up = right.cross(offset).normalize();
        self.position += right * self.pan_offset.x;
        self.position += up * self.pan_offset.y;
        self.target += right * self.pan_offset.x;
        self.target += up * self.pan_offset.y;

        // apply rotation
        offset = self.spherical.to_vec3();
        self.position = self.target + offset;

        // TODO?: if enable_damping...?
        self.spherical_delta = Spherical::from_vec3(Vector3::zero());

        self.scale = 1.0;
        self.pan_offset = Vector3::zero();

        // NOTE: skip zoomChanged stuff

        trace!("Position: {:?}\tTarget: {:?}\tfovy: {:?}", self.position, self.target, Deg(self.camera.fovy));
    }

    pub fn process_keyboard(&mut self, direction: CameraMovement, pressed: bool) {
        match direction {
            FORWARD => self.moving_forward = pressed,
            BACKWARD => self.moving_backward= pressed,
            LEFT => self.moving_left = pressed,
            RIGHT => self.moving_right = pressed,
        }
    }

    /// Do frame-based updates that require delta_time
    pub fn frame_update(&mut self, delta_time: f64) {
        let velocity = SPEED * delta_time as f32;

        let front = (self.target - self.position).normalize();
        if self.moving_forward {
            self.position += front * velocity;
            self.target += front * velocity;
        }
        if self.moving_backward {
            self.position += -(front * velocity);
            self.target += -(front * velocity);
        }

        let right = front.cross(Vector3::unit_y()).normalize();
        if self.moving_left {
            self.position += -(right * velocity);
            self.target += -(right * velocity);
        }
        if self.moving_right {
            self.position += right * velocity;
            self.target += right * velocity;
        }
    }

    pub fn set_camera(&mut self, camera: &Camera, transform: &Matrix4) {
        // spec: If no transformation is specified, the location of the camera is at the origin.
        let pos = transform * vec4(0.0, 0.0, 0.0, 1.0);

        // spec: ... the lens looks towards the local -Z axis ...
        let look_at = transform * vec4(0.0, 0.0, -1.0, 0.0);

        self.position = Point3::new(pos.x, pos.y, pos.z);
        self.target = Point3::new(look_at.x, look_at.y, look_at.z);

        // TODO!!: retaining current window aspect ratio for now... later maybe resize window accordingly?
        let mut camera = camera.clone();
        camera.update_aspect_ratio(self.camera.aspect_ratio());
        self.camera = camera;

        self.camera.update_projection_matrix();
    }
}
