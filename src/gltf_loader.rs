// use std::io::Read;

use gltf;
use gltf::Gltf;
// use gltf::accessor::{ Accessor };
// use gltf::buffer::{ Target };
// use gltf::mesh::{ Mode, Primitive };
use gltf::mesh::*;

// use gltf::import::Source;


pub fn load_file(path: &str) {
    let mut importer = gltf::Importer::new();
    let gltf = importer.import_from_path(path);
    match gltf {
        Ok(gltf) => {
            // println!("{:#?}", gltf);
            load_box(&gltf);
        }
        Err(err) => {
            println!("Error: {:#?}", err);
        }
    }
}

// struct PrimitiveData<'a> {
//     accessor: &'a Accessor<'a>,
//     buffer_view: &'a BufferView,
//     data: &'a [u8],
// }

pub fn load_box(gltf: &Gltf) {
    let buffer = &gltf.buffers().nth(0);

    let mesh = &gltf.meshes().nth(0).unwrap();
    let primitive = &mesh.primitives().nth(0).unwrap();

    let positions = primitive.position().unwrap();
    let normals = primitive.normal().unwrap();
    let indices = primitive.indices().unwrap();

    // TODO: No debug
    // println!("pos: {:?}", positions);
    println!("pos len: {}", positions.count());
    println!("nml len: {}", normals.count());
    match indices {
        Indices::U8(indices) => println!("idx8: {:?}", indices),
        Indices::U16(indices) => println!("idx16: {:?}", indices),
        Indices::U32(indices) => println!("idx32: {:?}", indices),
    }
    // TODO: No debug
    // assert_eq!(primitive.mode(), Mode::Triangles);

}
