use std::f32::consts::PI;

use cgmath::{vec3};
use cgmath::prelude::*;

use num_traits::clamp;

// type Point3 = cgmath::Point3<f32>;
// type Vector3 = cgmath::Vector3<f32>;
// type Matrix4 = cgmath::Matrix4<f32>;

use render::Camera;
use render::math::*;

// Defines several possible options for camera movement. Used as abstraction to stay away from window-system specific input methods
#[derive(PartialEq)]
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
const YAW: f32 = -90.0;
const PITCH: f32 = 0.0;
const SPEED: f32 = 2.5;
const SENSITIVTY: f32 = 0.1;
const ZOOM_SENSITIVITY: f32 = 0.1;
pub const ZOOM: f32 = 45.0;
const MIN_ZOOM: f32 = 1.0;
const MAZ_ZOOM: f32 = 90.0;

pub struct CameraControls {
    // Camera Attributes
    pub position: Point3,

    /// mutually exlusive: if center is set, it is used
    pub front: Vector3,
    pub center: Option<Point3>,

    pub up: Vector3,
    pub right: Vector3,
    pub world_up: Vector3,
    // Euler Angles
    pub yaw: f32,
    pub pitch: f32,
    // Camera options
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,

    pub camera: Camera,

    // pub moving_up: bool,
    pub moving_left: bool,
    // pub moving_down: bool,
    pub moving_right: bool,
    pub moving_forward: bool,
    pub moving_backward: bool,
}

impl Default for CameraControls {
    fn default() -> CameraControls {
        let mut controls = CameraControls {
            position: Point3::new(0.0, 0.0, 0.0),
            front: vec3(0.0, 0.0, -1.0),
            center: None,
            up: Vector3::zero(), // initialized later
            right: Vector3::zero(), // initialized later
            world_up: Vector3::unit_y(),
            yaw: YAW,
            pitch: PITCH,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVTY,

            camera: Camera::default(),

            // moving_up: false,
            moving_left: false,
            // moving_down: false,
            moving_right: false,
            moving_forward: false,
            moving_backward: false,
        };
        // TODO!!: overriding default order...? -> NO!
        controls.update_camera_vectors();
        controls
    }
}

impl CameraControls {
    pub fn camera_params(&self) -> CameraParams {
        CameraParams {
            position: self.position.to_vec(),
            view_matrix: self.view_matrix(),
            projection_matrix: self.camera.projection_matrix,
        }
    }

    // TODO!: cache? doesn't change every frame...
    /// Returns the view matrix calculated using Euler Angles and the LookAt Matrix
    pub fn view_matrix(&self) -> Matrix4 {
        if let Some(center) = self.center {
            Matrix4::look_at(self.position, center, self.up)
        }
        else {
            Matrix4::look_at(self.position, self.position + self.front, self.up)
        }
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
        if self.camera.fovy >= MIN_ZOOM && self.camera.fovy <= MAZ_ZOOM {
            self.camera.fovy -= yoffset;
        }
        if self.camera.fovy <= MIN_ZOOM {
            self.camera.fovy = MIN_ZOOM;
        }
        if self.camera.fovy >= MAZ_ZOOM {
            self.camera.fovy = MAZ_ZOOM;
        }
        self.camera.update_projection_matrix();
    }

    /// Calculates the front vector from the Camera's (updated) Eular Angles
    fn update_camera_vectors(&mut self) {
        if let Some(center) = self.center {
            self.front = center - self.position; // TODO!!!: overwritten again immediately...
            self.center = None;
        }
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

    pub fn set_camera(&mut self, camera: &Camera, transform: &Matrix4) {
        // spec: If no transformation is specified, the location of the camera is at the origin.
        let pos = transform * Vector4::zero();

        // spec: ... the lens looks towards the local -Z axis ...
        let look_at = transform * vec4(0.0, 0.0, -1.0, 0.0);

        self.position = Point3::new(pos.x, pos.y, pos.z);
        self.center = Some(Point3::new(look_at.x, look_at.y, look_at.z));

        // TODO!!: handle better (camera zoom/fovy)
        let mut camera = camera.clone();
        camera.fovy = self.camera.fovy;
        self.camera = camera;

        self.update_camera_vectors();
    }
}

/// Inspirted by ThreeJS OrbitControls
pub struct OrbitControls {
    pub camera: Camera,

    pub position: Point3,

    // "target" sets the location of focus, where the object orbits around
	pub target: Point3,

	// current position in spherical coordinates
	spherical: Spherical,
	spherical_delta: Spherical,

    scale: f32,
    pan_offset: Vector3,

    rotate_start: Vector2,
    rotate_end: Vector2,
    rotate_delta: Vector2,

    pub screen_width: f32,
    pub screen_height: f32,

    //
    offset: Vector3,

    quat: Quaternion,
    quat_inverse: Quaternion,

    last_position: Vector3,
    last_quaternion: Quaternion,
}

impl OrbitControls {
    pub fn new(position: Point3, screen_width: f32, screen_height: f32) -> Self {
        OrbitControls {
            camera: Camera::default(),

            position,
            target: Point3::new(0.0, 0.0, 0.0),

            // current position in spherical coordinates
            spherical: Spherical::default(),
            spherical_delta: Spherical::default(),

            scale: 1.0,
            pan_offset: Vector3::zero(),

            rotate_start: Vector2::zero(),
            rotate_end: Vector2::zero(),
            rotate_delta: Vector2::zero(),

            screen_width,
            screen_height,

            //
            offset: Vector3::zero(),

            // TODO!: works? different object.up? Quaternion::from_arc?
            quat: Quaternion::one(),
            quat_inverse: Quaternion::one(),

            last_position: Vector3::zero(),
            last_quaternion: Quaternion::zero(),
        }
    }

    pub fn camera_params(&self) -> CameraParams {
        CameraParams {
            position: self.position.to_vec(),
            view_matrix: self.view_matrix(),
            projection_matrix: self.camera.projection_matrix,
        }
    }

    pub fn view_matrix(&self) -> Matrix4 {
        Matrix4::look_at(self.position, self.target, vec3(0.0, 1.0, 0.0))
    }

    pub fn handle_mouse_move_rotate(&mut self, x: f32, y: f32) {
        self.rotate_end.x = x;
        self.rotate_end.y = y;
        self.rotate_delta = self.rotate_end - self.rotate_start;

        // rotating across whole screen goes 360 degrees around
        let rotate_speed = 1.0; // TODO: const/param/remove?
        let angle = 2.0 * PI * self.rotate_delta.x / self.screen_width * rotate_speed;
        self.rotate_left(angle);

        // rotating up and down along whole screen attempts to go 360, but limited to 180
        let angle = 2.0 * PI * self.rotate_delta.y / self.screen_height * rotate_speed;
		self.rotate_up(angle);

        self.rotate_start = self.rotate_end;

		self.update();
    }

    fn rotate_left(&mut self, angle: f32) {
        self.spherical_delta.theta -= angle;
    }

    fn rotate_up(&mut self, angle: f32) {
        self.spherical_delta.phi -= angle;
    }

    fn update(&mut self) {
        self.offset = self.position - self.target;

        // rotate offset to "y-axis-is-up" space
        self.offset = self.quat.rotate_vector(self.offset);

        // angle from z-axis around y-axis
        self.spherical = Spherical::from_vec3(self.offset);

        self.spherical.theta += self.spherical_delta.theta;
        self.spherical.phi += self.spherical_delta.phi;

        // TODO!: left out theta restrictions / make_safe for now

        // restrict phi to be between desired limits
        let epsilon = 0.0001;
        self.spherical.phi = clamp(self.spherical.phi, epsilon, PI - epsilon);

        self.spherical.radius *= self.scale;

        // TODO!: restrict radius to be between desired limits

        // move target to panned location
        self.target += self.pan_offset;

        self.offset = self.spherical.to_vec3();

        // rotate offset back to "camera-up-vector-is-up" space
        self.quat_inverse.rotate_vector(self.offset);

        self.position = self.target + self.offset;

        // TODO!!: how to do this?
        // scope.object.lookAt( scope.target );

        // TODO!: if enable_damping...?
        self.spherical_delta.radius = 0.0;
        self.spherical_delta.phi = 0.0;
        self.spherical_delta.theta = 0.0;

        self.scale = 1.0;
        self.pan_offset = Vector3::zero();

        // TODO!: zoomChanged stuff

        // TODO!!!
    }
}
