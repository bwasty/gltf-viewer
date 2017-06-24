#![allow(dead_code)]
extern crate cgmath;
// use cgmath::{Matrix4, vec3, Point3, Deg, perspective};
extern crate gl;
extern crate glfw;
// use self::glfw::{Context, Key, Action};
extern crate gltf;
extern crate image;

extern crate tobj; // TODO: TMP

// use std::sync::mpsc::Receiver;
// use std::ffi::CStr;

mod shader;
// use shader::Shader;
mod camera;
// use camera::Camera;
// use camera::Camera_Movement::*;
mod macros;
mod mesh;
mod model;
// use model::Model;

mod gltf_loader;
use gltf_loader::*;

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;


pub fn main() {
    load_file("src/data/Box.gltf");
    // load_file("src/data/minimal.gltf");
    // load_file("../gltf/glTF-Sample-Models/2.0/BoomBox/glTF/BoomBox.gltf");


}

