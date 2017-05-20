use std;
use std::io::Read;
use gltf::v2::{ import, Root };


pub fn load_file(path: &str) {
    let root = import(path);
    match root {
        Ok(root) => {
            println!("glTF version 2.0");
            println!("{:#?}", root);
            load_box(&root);
        }
        Err(err) => {
            println!("Error: {:#?}", err);
        }
    }
}

pub fn load_box(root: &Root) {
    let buffer = &root.buffers()[0];

    // TODO!: determine base directory...
    let mut file = std::fs::File::open(format!("src/data/{}", buffer.uri)).unwrap();
    let mut buffer_contents = Vec::with_capacity(buffer.byte_length as usize);
    file.read_to_end(&mut buffer_contents).unwrap();
    assert_eq!(buffer_contents.len(), buffer.byte_length as usize)
}
