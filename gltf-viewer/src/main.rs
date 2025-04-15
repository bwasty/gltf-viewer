// #![allow(dead_code)]
// #![allow(unused_features)]
// #![feature(test)]

use clap::{crate_version, value_parser, Arg, ArgAction, Command};

use log::warn;

use simplelog::{ColorChoice, ConfigBuilder as LogConfigBuilder, LevelFilter, TermLogger, TerminalMode};

mod viewer;
use crate::viewer::{CameraOptions, GltfViewer};

// TODO!!: math types..
pub use cgmath::{vec3, vec4};
pub type Vector3 = cgmath::Vector3<f32>;
use std::num::ParseFloatError;
pub fn parse_vec3(s: &str) -> Result<Vector3, ParseFloatError> {
    let coords: Vec<&str> = s.split(',').collect();
    assert!(coords.len() == 3, "Failed to parse Vector3 ({})", s);
    let x = coords[0].parse::<f32>()?;
    let y = coords[1].parse::<f32>()?;
    let z = coords[2].parse::<f32>()?;

    Ok(vec3(x, y, z))
}

pub fn main() {
    let args = Command::new("gltf-viewer")
        // TODO: version not display in --help anymore
        .version(option_env!("VERSION").unwrap_or(crate_version!()))
        .about("glTF 2.0 viewer\n\nNavigate with the mouse (left/right click + drag, mouse wheel) or WASD/cursor keys.")
        .arg(Arg::new("FILE") // TODO!: URL support?
            .required(true)
            .help("glTF file name"))
        .arg(Arg::new("verbose")
            .long("verbose")
            .short('v')
            .action(ArgAction::Count)
            .help("Enable verbose logging (log level INFO). Can be repeated up to 3 times to increase log level to DEBUG/TRACE)"))
        .arg(Arg::new("screenshot")
            .long("screenshot")
            .short('s')
            .value_name("FILE")
            .help("Create screenshot (PNG)"))
        .arg(Arg::new("WIDTH")
            .long("width")
            .short('W')
            .default_value("800")
            .help("Width in pixels")
            .value_parser(value_parser!(u32)))
        .arg(Arg::new("HEIGHT")
            .long("height")
            .short('H')
            .default_value("600")
            .help("Height in pixels")
            .value_parser(value_parser!(u32)))
        .arg(Arg::new("COUNT")
            .long("count")
            .short('c')
            .default_value("1")
            .help("Saves N screenshots of size WxH, rotating evenly spaced around the object")
            .value_parser(value_parser!(u32)))
        .arg(Arg::new("headless")
            .long("headless")
            .action(ArgAction::SetTrue)
            .help("Use real headless rendering for screenshots (default is a hidden window) [EXPERIMENTAL - see README for details]"))
        .arg(Arg::new("straight")
            .long("straight")
            .action(ArgAction::SetTrue)
            .help("Position camera in front of model if using default camera (i.e. glTF doesn't contain a camera or `--cam-index -1` is passed)"))
        .arg(Arg::new("scene")
            .long("scene")
            .default_value("0")
            .help("Index of the scene to load")
            .value_parser(value_parser!(u32)))
        .arg(Arg::new("CAM-INDEX")
            .long("cam-index")
            .default_value("0")
            .allow_negative_numbers(true)
            .help("Use the glTF camera with the given index (starting at 0). \n\
                Fallback if there is none: determine 'nice' camera position based on the scene's bounding box. \
                Can be forced by passing -1. \n\
                Note: All other camera options are ignored if this one is given.")
            .value_parser(value_parser!(i32)))
        .arg(Arg::new("CAM-POS")
            .long("cam-pos")
            .allow_negative_numbers(true)
            .help("Camera (aka eye) position override as comma-separated Vector3. Example: 1.2,3.4,5.6"))
        .arg(Arg::new("CAM-TARGET")
            .long("cam-target")
            .allow_negative_numbers(true)
            .help("Camera target (aka center) override as comma-separated Vector3. Example: 1.2,3.4,5.6"))
        .arg(Arg::new("CAM-FOVY")
            .long("cam-fovy")
            .default_value("75")
            .help("Vertical field of view ('zoom') in degrees.")
            .value_parser(value_parser!(u32)))
        .get_matches();

    let source = args.get_one::<String>("FILE").unwrap();

    let width = *args.get_one::<u32>("WIDTH").unwrap();
    let height = *args.get_one::<u32>("HEIGHT").unwrap();
    let count = *args.get_one::<u32>("COUNT").unwrap();

    let scene = *args.get_one::<u32>("scene").unwrap() as usize;

    let camera_options = CameraOptions {
        index: *args.get_one::<i32>("CAM-INDEX").unwrap(),
        // TODO!!: math types
        position: args.value_of("CAM-POS").map(|v| parse_vec3(v).unwrap()),
        target: args.value_of("CAM-TARGET").map(|v| parse_vec3(v).unwrap()),
        fovy: args
            .value_of("CAM-FOVY")
            .map(|n| Deg(n.parse().unwrap()))
            .unwrap(),
        straight: args.get_flag("straight"),
    };

    let log_level = match args.get_count("verbose") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let _ = TermLogger::init(
        log_level,
        LogConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .build(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    );

    let mut viewer = GltfViewer::new(
        source,
        width,
        height,
        args.get_flag("headless"),
        !args.contains_id("screenshot"),
        camera_options,
        scene,
    );

    if args.contains_id("screenshot") {
        let filename = args.get_one::<String>("screenshot").unwrap();

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

    // TODO!: start render loop
    // viewer.start_render_loop();
}

#[cfg(test)]
mod tests {
    use super::*;

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
