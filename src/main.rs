extern crate cgmath;
extern crate gltf;

mod gltf_loader;
use gltf_loader::*;

pub fn main() {
    load_file("src/data/Box.gltf");
    // load_file("src/data/minimal.gltf");
    // load_file("../gltf/glTF-Sample-Models/2.0/BoomBox/glTF/BoomBox.gltf");
}

