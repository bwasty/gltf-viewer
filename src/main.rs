// #![allow(dead_code)]
// #![allow(unused_features)]
// #![feature(test)]
#![allow(clippy::cast_lossless, clippy::cyclomatic_complexity)]

use clap::crate_version;
use cgmath;
use cgmath::Deg;

use clap::{Arg, App, AppSettings};

use log::warn;

use simplelog::{ColorChoice, TermLogger, LevelFilter, ConfigBuilder as
  LogConfigBuilder, TerminalMode};

mod utils;
mod viewer;
use crate::viewer::{GltfViewer, CameraOptions};

mod shader;
mod controls;
mod framebuffer;
mod macros;
mod importdata;
// TODO!: adapt Source...
// mod http_source;
// use http_source::HttpSource;
mod render;
use crate::render::math::*;

pub fn main() {
    let args = App::new("gltf-viewer")
        .version(option_env!("VERSION").unwrap_or(crate_version!()))
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::DeriveDisplayOrder)
        .before_help("glTF 2.0 viewer\n\nNavigate with the mouse (left/right click + drag, mouse wheel) \
                    or WASD/cursor keys.")
        .arg(Arg::with_name("FILE") // TODO!: re-add URL when fixed...
            .required(true)
            .takes_value(true)
            .help("glTF file name"))
        .arg(Arg::with_name("verbose")
            .long("verbose")
            .short("v")
            .multiple(true)
            .help("Enable verbose logging (log level INFO). Can be repeated up to 3 times to increase log level to DEBUG/TRACE)"))
        .arg(Arg::with_name("screenshot")
            .long("screenshot")
            .short("s")
            .value_name("FILE")
            .help("Create screenshot (PNG)"))
        .arg(Arg::with_name("WIDTH")
            .long("width")
            .short("w")
            .default_value("800")
            .help("Width in pixels")
            .validator(|value| value.parse::<u32>().map(|_| ()).map_err(|err| err.to_string())))
        .arg(Arg::with_name("HEIGHT")
            .long("height")
            .short("h")
            .default_value("600")
            .help("Height in pixels")
            .validator(|value| value.parse::<u32>().map(|_| ()).map_err(|err| err.to_string())))
        .arg(Arg::with_name("COUNT")
            .long("count")
            .short("c")
            .default_value("1")
            .help("Saves N screenshots of size WxH, rotating evenly spaced around the object")
            .validator(|value| value.parse::<u32>().map(|_| ()).map_err(|err| err.to_string())))
        .arg(Arg::with_name("headless")
            .long("headless")
            .help("Use real headless rendering for screenshots (default is a hidden window) [EXPERIMENTAL - see README for details]"))
        .arg(Arg::with_name("straight")
            .long("straight")
            .help("Position camera in front of model if using default camera (i.e. glTF doesn't contain a camera or `--cam-index -1` is passed)"))
        .arg(Arg::with_name("scene")
            .long("scene")
            .default_value("0")
            .help("Index of the scene to load")
            .validator(|value| value.parse::<u32>().map(|_| ()).map_err(|err| err.to_string())))
        .arg(Arg::with_name("CAM-INDEX")
            .long("cam-index")
            .takes_value(true)
            .default_value("0")
            .allow_hyphen_values(true)
            .help("Use the glTF camera with the given index (starting at 0). \n\
                Fallback if there is none: determine 'nice' camera position based on the scene's bounding box. \
                Can be forced by passing -1. \n\
                Note: All other camera options are ignored if this one is given.")
            .validator(|value| value.parse::<i32>().map(|_| ()).map_err(|err| err.to_string())))
        .arg(Arg::with_name("CAM-POS")
            .long("cam-pos")
            .takes_value(true)
            .allow_hyphen_values(true)
            .help("Camera (aka eye) position override as comma-separated Vector3. Example: 1.2,3.4,5.6"))
        .arg(Arg::with_name("CAM-TARGET")
            .long("cam-target")
            .takes_value(true)
            .allow_hyphen_values(true)
            .help("Camera target (aka center) override as comma-separated Vector3. Example: 1.2,3.4,5.6"))
        .arg(Arg::with_name("CAM-FOVY")
            .long("cam-fovy")
            .takes_value(true)
            .default_value("75")
            .help("Vertical field of view ('zoom') in degrees.")
            .validator(|value| value.parse::<u32>().map(|_| ()).map_err(|err| err.to_string())))
        .get_matches();
    let source = args.value_of("FILE").unwrap();

    let width: u32 = args.value_of("WIDTH").unwrap().parse().unwrap();
    let height: u32 = args.value_of("HEIGHT").unwrap().parse().unwrap();
    let count: u32 = args.value_of("COUNT").unwrap().parse().unwrap();

    let scene: usize = args.value_of("scene").unwrap().parse().unwrap();

    let camera_options = CameraOptions {
        index: args.value_of("CAM-INDEX").map(|n| n.parse().unwrap()).unwrap(),
        position: args.value_of("CAM-POS").map(|v| parse_vec3(v).unwrap()),
        target: args.value_of("CAM-TARGET").map(|v| parse_vec3(v).unwrap()),
        fovy: args.value_of("CAM-FOVY").map(|n| Deg(n.parse().unwrap())).unwrap(),
        straight: args.is_present("straight"),
    };

    let log_level = match args.occurrences_of("verbose") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace
    };

    let _ = TermLogger::init(
        log_level,
        LogConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .build(),
        TerminalMode::Stdout,
        ColorChoice::Auto);

    let mut event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut viewer = GltfViewer::new(&mut event_loop, source, width, height,
        args.is_present("headless"),
        !args.is_present("screenshot"),
        camera_options,
        scene);

    if args.is_present("screenshot") {
        let filename = args.value_of("screenshot").unwrap();

        if !filename.to_lowercase().ends_with(".png") {
            warn!("filename should end with .png");
        }
        if count > 1 {
            viewer.multiscreenshot(filename, count)
        } else {
            viewer.screenshot(filename)
        }
        return;
    }

    event_loop.run_app(&mut viewer).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_struct_sizes() {
        // run with `cargo test -- --nocapture`
        println!("Sizes in bytes:");
        println!("Scene:     {:>3}", std::mem::size_of::<render::Scene>());
        println!("Node:      {:>3}", std::mem::size_of::<render::Node>());
        println!("Mesh:      {:>3}", std::mem::size_of::<render::Mesh>());
        println!("Primitive: {:>3}", std::mem::size_of::<render::Primitive>());
        println!("Vertex:    {:>3}", std::mem::size_of::<render::Vertex>());
        println!();
        println!("Option<String>: {:>3}", std::mem::size_of::<Option<String>>());
        println!("String:         {:>3}", std::mem::size_of::<String>());
        println!("Vec<f32>:       {:>3}", std::mem::size_of::<Vec<f32>>());
        println!("Vec<Node>:      {:>3}", std::mem::size_of::<Vec<render::Node>>());
    }

//     extern crate test;
//     use self::test::Bencher;
//     #[bench]
//     fn bench_frame_timer(b: &mut Bencher) {
//         let mut timer = FrameTimer::new("Foobar", 60);
//         b.iter(|| {
//             for _ in 0..60 {
//                 timer.start();
//                 timer.end();
//             }
//         })
//     }
}
