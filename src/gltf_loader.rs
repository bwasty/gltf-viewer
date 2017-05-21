use std;
use std::io::Read;
use gltf::v2::{ import, Root };
use gltf::v2::accessor::ComponentType;
use gltf::v2::buffer::Target;
use gltf::v2::mesh::Mode;

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
    assert_eq!(buffer_contents.len(), buffer.byte_length as usize); 

    let mesh = &root.meshes()[0];
    let primitive = &mesh.primitives[0];
    assert_eq!(primitive.mode, Mode::Triangles);

    let posAccessorIndex = primitive.attributes["POSITION"].value() as usize;
    let posAccessor = &root.accessors()[posAccessorIndex];
    let posBufferView = &root.buffer_views()[posAccessor.buffer_view.value() as usize];

    let positionData = &buffer_contents[
        posBufferView.byte_offset as usize .. (posBufferView.byte_offset + posBufferView.byte_length) as usize];

}
