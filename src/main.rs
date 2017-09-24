#![allow(dead_code)]
#![allow(unknown_lints)]
// #![allow(unused_features)]
// #![feature(test)]
#[macro_use] extern crate clap;
extern crate cgmath;
// use cgmath::prelude::*;

extern crate gl;

extern crate glutin;

extern crate gltf;
extern crate gltf_importer;

extern crate gltf_utils;
extern crate image;

#[macro_use]
extern crate bitflags;

use clap::{Arg, App, AppSettings};


#[macro_use]extern crate log;
extern crate simplelog;
use simplelog::{TermLogger, LogLevelFilter, Config as LogConfig};

mod utils;
mod viewer;
use viewer::{GltfViewer};

mod shader;
mod camera;
mod framebuffer;
mod macros;
// TODO!: adapt Source...
// mod http_source;
// use http_source::HttpSource;
mod render;

pub fn main() {
    let args = App::new("gltf-viewer")
        .version(crate_version!())
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::DeriveDisplayOrder)
        .arg(Arg::with_name("FILE") // TODO!: re-add URL when fixed...
            .required(true)
            .takes_value(true)
            .help("glTF file name"))
        .arg(Arg::with_name("screenshot")
            .long("screenshot")
            .short("s")
            .value_name("FILE")
            .help("Create screenshot (PNG)"))
        .arg(Arg::with_name("verbose")
            .long("verbose")
            .short("v")
            .multiple(true)
            .help("Enable verbose logging (log level INFO). Can be repeated multiple times to increase log level to DEBUG/TRACE)"))
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
        .get_matches();
    let source = args.value_of("FILE").unwrap();
    let width: u32 = args.value_of("WIDTH").unwrap().parse().unwrap();
    let height: u32 = args.value_of("HEIGHT").unwrap().parse().unwrap();

    let log_level = match args.occurrences_of("verbose") {
        0 => LogLevelFilter::Warn,
        1 => LogLevelFilter::Info,
        2 => LogLevelFilter::Debug,
        _ => LogLevelFilter::Trace
    };

    let _ = TermLogger::init(log_level, LogConfig { time: None, target: None, ..LogConfig::default() });

    // TODO!: headless rendering doesn't work (only clearcolor)
    let mut viewer = GltfViewer::new(source, width, height,
        // args.is_present("screenshot")
        false,
        !args.is_present("screenshot")
    );

    if args.is_present("screenshot") {
        let filename = args.value_of("screenshot").unwrap();
        if !filename.to_lowercase().ends_with(".png") {
            warn!("filename should end with .png");
        }
        viewer.screenshot(filename, width, height);
        return;
    }

    viewer.start_render_loop();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_struct_sizes() {
        // run with `cargo test -- --nocapture`
        println!("Sizes in bytes:");
        println!("Scene:     {:>3}", std::mem::size_of::<Scene>());
        println!("Node:      {:>3}", std::mem::size_of::<Node>());
        println!("Mesh:      {:>3}", std::mem::size_of::<Mesh>());
        println!("Primitive: {:>3}", std::mem::size_of::<Primitive>());
        println!("Vertex:    {:>3}", std::mem::size_of::<Vertex>());
        println!();
        println!("Option<String>: {:>3}", std::mem::size_of::<Option<String>>());
        println!("String:         {:>3}", std::mem::size_of::<String>());
        println!("Vec<f32>:       {:>3}", std::mem::size_of::<Vec<f32>>());
        println!("Vec<Node>:      {:>3}", std::mem::size_of::<Vec<Node>>());
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
