use std::path::Path;
use std::rc::Rc;

// TODO timing for wasm target
#[cfg(not(feature = "use_wasm_bindgen"))]
use std::time::Instant;

use cgmath::{ Deg, Point3 };
use collision::Aabb;

use gltf;

#[cfg(feature = "use_wasm_bindgen")]
use crate::{error, warn, info};
#[cfg(not(feature = "use_wasm_bindgen"))]
use log::{error, warn, info};

#[cfg(feature = "use_wasm_bindgen")]
use web_sys::WebGl2RenderingContext;


use crate::controls::{OrbitControls, ScreenSize};
use crate::importdata::ImportData;
use crate::render::*;
use crate::render::math::*;
#[cfg(not(feature = "use_wasm_bindgen"))]
use crate::utils::{print_elapsed, FrameTimer};

use crate::platform::*;

// TODO!: complete and pass through draw calls? or get rid of multiple shaders?
// How about state ordering anyway?
// struct DrawState {
//     current_shader: ShaderFlags,
//     back_face_culling_enabled: bool
// }

#[derive(Copy, Clone)]
pub struct CameraOptions {
    pub index: i32,
    pub position: Option<Vector3>,
    pub target: Option<Vector3>,
    pub fovy: Deg<f32>,
    pub straight: bool,
}

pub struct GltfViewer {
    renderer: GltfViewerRenderer,
    orbit_controls: OrbitControls,

    // TODO!: get rid of scene?
    root: Option<Root>,
    scene: Option<Scene>,

    // TODO timing for wasm target
    #[cfg(not(feature = "use_wasm_bindgen"))]
    delta_time: f64, // seconds
    #[cfg(not(feature = "use_wasm_bindgen"))]
    last_frame: Instant,

    #[cfg(not(feature = "use_wasm_bindgen"))]
    render_timer: FrameTimer,
}

/// Note about `headless` and `visible`: True headless rendering doesn't work on
/// all operating systems, but an invisible window usually works
impl GltfViewer {
    #[cfg(not(feature = "use_wasm_bindgen"))]
    pub fn new(
        width: u32,
        height: u32,
        headless: bool,
        visible: bool,
        camera_options: &CameraOptions,
    ) -> GltfViewer {
        // create renderer to wrap gl
        let renderer = GltfViewerRenderer::new(width,height,headless,visible);
        GltfViewer::new_from_renderer(headless, visible, camera_options, renderer)
    }

    /// constructor for webgl contexts
    #[cfg(feature = "use_wasm_bindgen")]
    pub fn new_from_webgl(
        gl: Rc<WebGl2RenderingContext>,
        width: u32,
        height: u32,
        headless: bool,
        visible: bool,
        camera_options: &CameraOptions,
    ) -> GltfViewer {
        // create renderer to wrap webgl
        let renderer = GltfViewerRenderer::new(gl,width,height);
        GltfViewer::new_from_renderer(headless, visible, camera_options, renderer)
    }

    pub fn new_from_renderer(
        headless: bool,
        visible: bool,
        camera_options: &CameraOptions,
        renderer: GltfViewerRenderer,
    ) -> GltfViewer {
        let mut orbit_controls = OrbitControls::new(
            Point3::new(0.0, 0.0, 2.0),
            ScreenSize::clone(&renderer.size));
        orbit_controls.camera = Camera::default();
        orbit_controls.camera.fovy = camera_options.fovy;
        orbit_controls.camera.update_aspect_ratio(renderer.size.width / renderer.size.height); // updates projection matrix

        renderer.init_viewer_gl_context(headless, visible);

        GltfViewer {
            renderer,

            orbit_controls,

            root: None,
            scene: None,

            // TODO timing for wasm target
            #[cfg(not(feature = "use_wasm_bindgen"))]
            delta_time: 0.0, // seconds
            #[cfg(not(feature = "use_wasm_bindgen"))]
            last_frame: Instant::now(),

            #[cfg(not(feature = "use_wasm_bindgen"))]
            render_timer: FrameTimer::new("rendering", 300),
        }
    }

    pub fn load(&mut self, source: &str, scene_index: usize, camera_options: &CameraOptions) {
        // TODO timing for wasm target
        // let mut start_time = Instant::now();
        
        // TODO!: http source
        // let gltf =
        if source.starts_with("http") {
            panic!("not implemented: HTTP support temporarily removed.")
            // let http_source = HttpSource::new(source);
            // let import = gltf::Import::custom(http_source, Default::default());
            // let gltf = import_gltf(import);
            // println!(); // to end the "progress dots"
            // gltf
        }
        //     else {
        let (doc, buffers, images) = match gltf::import(source) {
            Ok(tuple) => tuple,
            Err(err) => {
                error!("glTF import failed: {:?}", err);
                if let gltf::Error::Io(_) = err {
                    error!("Hint: Are the .bin file(s) referenced by the .gltf file available?")
                }
                panic!("gltf file not found")
            },
        };
        let imp = ImportData { doc, buffers, images };
        self.process_import_data(source, imp, scene_index);
        self.update_camera_from_scene(camera_options);
    }

    pub fn load_from_bytes(&mut self, source_bytes: &[u8], source_name: &str, scene_index: usize, camera_options: &CameraOptions) {
        // TODO timing for wasm target
        // let mut start_time = Instant::now();
        
        let (doc, buffers, images) = match gltf::import_slice(source_bytes) {
            Ok(tuple) => tuple,
            Err(err) => {
                error!("glTF import failed: {:?}", err);
                if let gltf::Error::Io(_) = err {
                    error!("Hint: Are the .bin file(s) referenced by the .gltf file available?")
                }
                panic!("gltf file not found")
            },
        };

        let imp = ImportData { doc, buffers, images };
        self.process_import_data(source_name, imp, scene_index);
        self.update_camera_from_scene(camera_options);
    }

    fn process_import_data(&mut self, source: &str, imp: ImportData, scene_index: usize) {
        // TODO timing for wasm target
        // print_elapsed("Imported glTF in ", start_time);
        // start_time = Instant::now();

        // load first scene
        if scene_index >= imp.doc.scenes().len() {
            error!("Scene index too high - file has only {} scene(s)", imp.doc.scenes().len());
            panic!("gltf scene index too high")
        }
        let base_path = Path::new(source);
        let mut root = Root::from_gltf(&imp, base_path, &mut self.renderer);
        let scene = Scene::from_gltf(&imp.doc.scenes().nth(scene_index).unwrap(), &mut root);
        
        // TODO timing for wasm target
        // print_elapsed(&format!("Loaded scene with {} nodes, {} meshes in ",
        //         imp.doc.nodes().count(), imp.doc.meshes().len()), start_time);

        self.root = Some(root);
        self.scene = Some(scene);


        // check for gl errors
        #[cfg(not(feature = "use_wasm_bindgen"))]
        unsafe { crate::platform::gl::utils::gl_check_error(); };
    }

    fn update_camera_from_scene(&mut self, camera_options: &CameraOptions) {
        if camera_options.index != 0 && camera_options.index >= self.root.as_ref().unwrap().camera_nodes.len() as i32 {
            error!("No camera with index {} found in glTF file (max: {})",
                camera_options.index, self.root.as_ref().unwrap().camera_nodes.len() as i32 - 1);
            panic!("Process Exiting")
        }
        if !self.root.as_ref().unwrap().camera_nodes.is_empty() && camera_options.index != -1 {
            let cam_node = &self.root.as_ref().unwrap().get_camera_node(camera_options.index as usize);
            let cam_node_info = format!("{} ({:?})", cam_node.index, cam_node.name);
            let cam = cam_node.camera.as_ref().unwrap();
            info!("Using camera {} on node {}", cam.description(), cam_node_info);
            self.orbit_controls.set_camera(cam, &cam_node.final_transform);

            if camera_options.position.is_some() || camera_options.target.is_some() {
                warn!("Ignoring --cam-pos / --cam-target since --cam-index is given.")
            }
        } else {
            info!("Determining camera view from bounding box");
            self.set_camera_from_bounds(camera_options.straight);

            if let Some(p) = camera_options.position {
                self.orbit_controls.position = Point3::from_vec(p)
            }
            if let Some(target) = camera_options.target {
                self.orbit_controls.target = Point3::from_vec(target)
            }
        }
    }

    /// determine "nice" camera perspective from bounding box. Inspired by donmccurdy/three-gltf-viewer
    fn set_camera_from_bounds(&mut self, straight: bool) {
        let bounds = &self.scene.as_ref().unwrap().bounds;
        let size = (bounds.max - bounds.min).magnitude();
        let center = bounds.center();

        // TODO: x,y addition optional
        let cam_pos = if straight {
            Point3::new(
                center.x,
                center.y,
                center.z + size * 0.75,
            )
        } else {
            Point3::new(
                center.x + size / 2.0,
                center.y + size / 5.0,
                center.z + size / 2.0,
            )
        };

        self.orbit_controls.position = cam_pos;
        self.orbit_controls.target = center;
        self.orbit_controls.camera.znear = size / 100.0;
        self.orbit_controls.camera.zfar = Some(size * 20.0);
        self.orbit_controls.camera.update_projection_matrix();
    }

    #[cfg(not(feature = "use_wasm_bindgen"))]
    pub fn start_render_loop(&mut self) {
        loop {
            // per-frame time logic
            // NOTE: Deliberately ignoring the seconds of `elapsed()`
            self.delta_time = f64::from(self.last_frame.elapsed().subsec_nanos()) / 1_000_000_000.0;
            self.last_frame = Instant::now();

            // render frame
            let keep_running = self.render_frame(self.delta_time);
            
            // check if program end
            if !keep_running {

                // final error check so errors don't go unnoticed
                #[cfg(not(feature = "use_wasm_bindgen"))]
                unsafe { crate::platform::gl::utils::gl_check_error(); };

                break
            }
        }
    }

    /// Render frame with orbit controls camera
    pub fn render_frame(&mut self, delta_time: f64) -> bool {
        // events
        let keep_running = self.renderer.process_events(&mut self.orbit_controls);

        self.orbit_controls.frame_update(delta_time); // keyboard navigation

        #[cfg(not(feature = "use_wasm_bindgen"))]
        self.render_timer.start();

        self.renderer.draw(self.scene.as_mut().unwrap(), self.root.as_mut().unwrap(), &self.orbit_controls);

        #[cfg(not(feature = "use_wasm_bindgen"))]
        self.render_timer.end();

        keep_running
    }

    pub fn screenshot(&mut self, filename: &str) {
        self.renderer.screenshot(self.scene.as_mut().unwrap(), self.root.as_mut().unwrap(), &mut self.orbit_controls, filename);
    }
    pub fn multiscreenshot(&mut self, filename: &str, count: u32) {
        self.renderer.multiscreenshot(self.scene.as_mut().unwrap(), self.root.as_mut().unwrap(), &mut self.orbit_controls, filename, count);
    }
}
