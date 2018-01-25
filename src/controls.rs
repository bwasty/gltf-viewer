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
const MAZ_ZOOM: f32 = 170.0;

/// NOTE: superceded by OrbitControls. Keeping the parts of it not (yet?) added there.
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
        let controls = CameraControls {
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
        // controls.update_camera_vectors();
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

        // self.update_camera_vectors();
    }
}

#[derive(Clone)]
pub enum NavState {
    None,
    Rotating,
    Panning,
}

/// Inspirted by ThreeJS OrbitControls
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

    pub screen_width: f32,
    pub screen_height: f32,
}

impl OrbitControls {
    pub fn new(position: Point3, screen_width: f32, screen_height: f32) -> Self {
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

            screen_width,
            screen_height,
        }
    }

    // TODO!!: cache/return reference? often many of them stay the same...
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

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        match self.state {
            NavState::Rotating => self.handle_mouse_move_rotate(x, y),
            NavState::Panning => self.handle_mouse_move_pan(x, y),
            NavState::None => ()
        }
    }

    fn handle_mouse_move_rotate(&mut self, x: f32, y: f32) {
        self.rotate_end.x = x;
        self.rotate_end.y = y;
        let rotate_delta = if let Some(rotate_start) = self.rotate_start {
            self.rotate_end - rotate_start
        } else {
            Vector2::zero()
        };

        // rotating across whole screen goes 360 degrees around
        let rotate_speed = 1.0; // TODO: const/param/remove?
        let angle = 2.0 * PI * rotate_delta.x / self.screen_width * rotate_speed;
        self.rotate_left(angle);
        println!("{:?}",angle);

        // rotating up and down along whole screen attempts to go 360, but limited to 180
        let angle = 2.0 * PI * rotate_delta.y / self.screen_height * rotate_speed;
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
    pub fn angle_me(&mut self, angle: f32) {
        self.rotate_left(angle);
        self.update();
    }
    fn rotate_up(&mut self, angle: f32) {
        self.spherical_delta.phi -= angle;
    }

    fn handle_mouse_move_pan(&mut self, x: f32, y: f32) {
        self.pan_end.x = x;
        self.pan_end.y = y;

        let pan_delta = if let Some(pan_start) = self.pan_start {
            self.pan_end - pan_start
        } else {
            Vector2::zero()
        };

        self.pan(&pan_delta);

        self.pan_start = Some(self.pan_end);

        self.update();
    }

    fn pan(&mut self, delta: &Vector2) {
        if self.camera.is_perspective() {
            let offset = self.position - self.target;
            let mut target_distance = offset.magnitude();

            // half of the fov is center to top of screen
            target_distance *= (self.camera.fovy / 2.0).tan() * PI / 180.0;

            // we actually don't use screen_width, since perspective camera is fixed to screen height
            let distance = 2.0 * delta.x * target_distance / self.screen_height;
            self.pan_left(distance);
            let distance = 2.0 * delta.y * target_distance / self.screen_height;
            self.pan_up(distance);
        } else {
            // TODO!: orthographic camera pan
            unimplemented!("orthographic camera pan")
        }
    }

    pub fn pan_left(&mut self, distance: f32) {
        self.pan_offset.x -= distance
    }

    pub fn pan_up(&mut self, distance: f32) {
        self.pan_offset.y -= distance
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
    }
}
