// #![allow(dead_code)]
// #![allow(unused_features)]
// #![feature(test)]

use clap::{crate_version, value_parser, Arg, ArgAction, Command};

use log::warn;

use simplelog::{ColorChoice, ConfigBuilder as LogConfigBuilder, LevelFilter, TermLogger, TerminalMode};

mod viewer;
use crate::viewer::{CameraOptions, GltfViewer};

use bevy::prelude::Vec3;
use std::num::ParseFloatError;
pub fn parse_vec3(s: &str) -> Result<Vec3, ParseFloatError> {
    let coords: Vec<&str> = s.split(',').collect();
    assert!(coords.len() == 3, "Failed to parse Vec3 ({})", s);
    let x = coords[0].parse::<f32>()?;
    let y = coords[1].parse::<f32>()?;
    let z = coords[2].parse::<f32>()?;

    Ok(Vec3::new(x, y, z))
}

pub fn build_cli() -> Command {
    Command::new("gltf-viewer")
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
            .value_name("X,Y,Z")
            .help("Camera (aka eye) position override as comma-separated Vector3. Example: 1.2,3.4,5.6"))
        .arg(Arg::new("CAM-TARGET")
            .long("cam-target")
            .allow_negative_numbers(true)
            .value_name("X,Y,Z")
            .help("Camera target (aka center) override as comma-separated Vector3. Example: 1.2,3.4,5.6"))
        .arg(Arg::new("CAM-FOVY")
            .long("cam-fovy")
            .default_value("75")
            .help("Vertical field of view ('zoom') in degrees.")
            .value_parser(value_parser!(u32)))
}

pub struct AppConfig {
    pub source: String,
    pub width: u32,
    pub height: u32,
    pub count: u32,
    pub scene: usize,
    pub camera_options: CameraOptions,
    pub log_level: LevelFilter,
    pub headless: bool,
    pub screenshot: Option<String>,
}

pub fn parse_args(args: clap::ArgMatches) -> AppConfig {
    let source = args.get_one::<String>("FILE").unwrap().clone();

    let width = *args.get_one::<u32>("WIDTH").unwrap();
    let height = *args.get_one::<u32>("HEIGHT").unwrap();
    let count = *args.get_one::<u32>("COUNT").unwrap();

    let scene = *args.get_one::<u32>("scene").unwrap() as usize;

    let camera_options = CameraOptions {
        index: *args.get_one::<i32>("CAM-INDEX").unwrap(),
        position: args
            .get_one::<String>("CAM-POS")
            .map(|v| parse_vec3(v).unwrap()),
        target: args
            .get_one::<String>("CAM-TARGET")
            .map(|v| parse_vec3(v).unwrap()),
        fovy: *args.get_one::<u32>("CAM-FOVY").unwrap(),
        straight: args.get_flag("straight"),
    };

    let log_level = match args.get_count("verbose") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let headless = args.get_flag("headless");
    let screenshot = if args.contains_id("screenshot") {
        Some(args.get_one::<String>("screenshot").unwrap().clone())
    } else {
        None
    };

    AppConfig {
        source,
        width,
        height,
        count,
        scene,
        camera_options,
        log_level,
        headless,
        screenshot,
    }
}

pub fn main() {
    let args = build_cli().get_matches();
    let config = parse_args(args);

    let _ = TermLogger::init(
        config.log_level,
        LogConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .build(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    );

    let mut viewer = GltfViewer::new(
        &config.source,
        config.width,
        config.height,
        config.headless,
        config.screenshot.is_none(),
        config.camera_options,
        config.scene,
    );

    if let Some(filename) = config.screenshot.as_ref() {
        if !filename.to_lowercase().ends_with(".png") {
            warn!("filename should end with .png");
        }
        if config.count > 1 {
            viewer.multiscreenshot(filename, config.count)
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

    fn build_config_from_args(args: Vec<&str>) -> AppConfig {
        let matches = build_cli()
            .try_get_matches_from(args)
            .expect("Failed to parse args");
        parse_args(matches)
    }

    #[test]
    fn test_parse_vec3() {
        let result = parse_vec3("1.0,2.0,3.0").unwrap();
        assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));

        // Test parsing with negative values
        let result = parse_vec3("-1.5,-2.5,-3.5").unwrap();
        assert_eq!(result, Vec3::new(-1.5, -2.5, -3.5));
    }

    #[test]
    #[should_panic(expected = "Failed to parse Vec3")]
    fn test_parse_vec3_invalid_format() {
        // Test with too few components
        parse_vec3("1.0,2.0").unwrap();
    }

    #[test]
    fn test_parse_vec3_invalid_number() {
        // Test with invalid number format
        let result = parse_vec3("1.0,invalid,3.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_default_config() {
        // Test with just the required FILE arg
        let config = build_config_from_args(vec!["gltf-viewer", "model.gltf"]);

        // Check default values
        assert_eq!(config.source, "model.gltf");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.count, 1);
        assert_eq!(config.scene, 0);
        assert_eq!(config.camera_options.index, 0);
        assert_eq!(config.camera_options.fovy, 75);
        assert_eq!(config.camera_options.position, None);
        assert_eq!(config.camera_options.target, None);
        assert_eq!(config.camera_options.straight, false);
        assert_eq!(config.log_level, LevelFilter::Warn);
        assert_eq!(config.headless, false);
        assert_eq!(config.screenshot, None);
    }

    #[test]
    fn test_config_with_camera_options() {
        // Test with camera positioning options
        let config = build_config_from_args(vec![
            "gltf-viewer",
            "model.gltf",
            "--cam-pos",
            "1.0,2.0,3.0",
            "--cam-target",
            "4.0,5.0,6.0",
            "--cam-fovy",
            "60",
            "--straight",
        ]);

        // Check camera options
        assert_eq!(
            config.camera_options.position,
            Some(Vec3::new(1.0, 2.0, 3.0))
        );
        assert_eq!(config.camera_options.target, Some(Vec3::new(4.0, 5.0, 6.0)));
        assert_eq!(config.camera_options.fovy, 60);
        assert_eq!(config.camera_options.straight, true);
    }

    #[test]
    fn test_negative_camera_index() {
        // Test with negative camera index (special case that forces fallback)
        let config = build_config_from_args(vec!["gltf-viewer", "model.gltf", "--cam-index", "-1"]);

        assert_eq!(config.camera_options.index, -1);
    }

    #[test]
    fn test_config_screenshot_option() {
        // Test screenshot option
        let config = build_config_from_args(vec![
            "gltf-viewer",
            "model.gltf",
            "--screenshot",
            "output.png",
            "--count",
            "5",
            "--headless",
        ]);

        assert_eq!(config.screenshot, Some("output.png".to_string()));
        assert_eq!(config.count, 5);
        assert_eq!(config.headless, true);
    }

    #[test]
    fn test_config_short_flags() {
        // Test short flag versions
        let config = build_config_from_args(vec![
            "gltf-viewer",
            "model.gltf",
            "-s",
            "screenshot.png",
            "-W",
            "1024",
            "-H",
            "768",
            "-c",
            "3",
            "-v",
        ]);

        assert_eq!(config.screenshot, Some("screenshot.png".to_string()));
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.count, 3);
        assert_eq!(config.log_level, LevelFilter::Info); // One -v = Info
    }

    #[test]
    fn test_config_verbose_levels() {
        let expected_levels = [
            LevelFilter::Warn,  // 0 flags
            LevelFilter::Info,  // 1 flag
            LevelFilter::Debug, // 2 flags
            LevelFilter::Trace, // 3+ flags
        ];

        // Test each verbosity level
        for (i, expected_level) in expected_levels.iter().enumerate() {
            let mut args = vec!["gltf-viewer", "model.gltf"];
            let mut v_args = vec![];
            for _ in 0..i {
                v_args.push("-v");
            }
            args.extend(v_args);

            let config = build_config_from_args(args);
            assert_eq!(
                config.log_level, *expected_level,
                "Verbosity level {} incorrect",
                i
            );
        }
    }

    #[test]
    fn test_config_dimensions() {
        // Test custom dimensions
        let config = build_config_from_args(vec![
            "gltf-viewer",
            "model.gltf",
            "--width",
            "1920",
            "--height",
            "1080",
        ]);

        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn test_scene_selection() {
        // Test scene selection
        let config = build_config_from_args(vec!["gltf-viewer", "model.gltf", "--scene", "2"]);

        assert_eq!(config.scene, 2);
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
