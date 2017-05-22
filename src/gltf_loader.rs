use std;
use std::io::Read;
use gltf::v2::{ import, Root };
use gltf::v2::accessor::{ Accessor, ComponentType };
use gltf::v2::buffer::{ Target, BufferView};
use gltf::v2::mesh::{ Mode, Primitive };

pub fn load_file(path: &str) {
    let root = import(path);
    match root {
        Ok(root) => {
            println!("glTF version 2.0");
            // println!("{:#?}", root);
            load_box(&root);
        }
        Err(err) => {
            println!("Error: {:#?}", err);
        }
    }
}

struct PrimitiveData<'a> {
    accessor: &'a Accessor,
    buffer_view: &'a BufferView,
    data: &'a [u8],
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

    let pos_accessor_index = primitive.attributes["POSITION"].value() as usize;
    let pos_accessor = &root.accessors()[pos_accessor_index];
    let pos_buffer_view = &root.buffer_views()[pos_accessor.buffer_view.value() as usize];

    let position_data = &buffer_contents[
        pos_buffer_view.byte_offset as usize .. (pos_buffer_view.byte_offset + pos_buffer_view.byte_length) as usize];

    // TODO!: deal with no index case
    let index_accessor_index = primitive.indices.as_ref().unwrap().clone();
    let index_accessor = &root.accessor(index_accessor_index);
    let index_buffer_view = &root.buffer_view(index_accessor.buffer_view.clone());
    let index_data = &buffer_contents[
        index_buffer_view.byte_offset as usize .. (index_buffer_view.byte_offset + index_buffer_view.byte_length) as usize];
    
    let normals_accessor_index = primitive.attributes["NORMAL"].clone();
    let normals_accessor = &root.accessor(normals_accessor_index);
    let normal_buffer_view = &root.buffer_view(index_accessor.buffer_view.clone());
    let normal_data = &buffer_contents[
        normal_buffer_view.byte_offset as usize .. (normal_buffer_view.byte_offset + normal_buffer_view.byte_length) as usize
    ];

    println!("pos len: {}", position_data.len());
    println!("idx len: {}", index_data.len());
}

// fn get_attribute_data(root: &Root, buffer_data: &[u8], primitive: &Primitive, attribute_name: &str) {
//     let accessor = &root.accessor(primitive)
// }