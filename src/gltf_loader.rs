// use std::io::Read;
use cgmath::Vector3;

use gltf;
use gltf::Gltf;
// use gltf::accessor::{ Accessor };
// use gltf::buffer::{ Target };
// use gltf::mesh::{ Mode, Primitive };
use gltf::mesh::*;

use mesh::{ Vertex, Mesh as RenderMesh };

// use gltf::import::Source;


pub fn load_file(path: &str) -> RenderMesh {
    let mut importer = gltf::Importer::new();
    let gltf = importer.import_from_path(path);
    match gltf {
        Ok(gltf) => {
            // println!("{:#?}", gltf);
            return load_box(&gltf)
        }
        Err(err) => {
            println!("Error: {:?}", err);
            panic!();
        }
    }
}

pub fn load_box(gltf: &Gltf) -> RenderMesh {
    let mesh = &gltf.meshes().nth(0).unwrap();
    let primitive = &mesh.primitives().nth(0).unwrap();

    let positions = primitive.position().unwrap();
    let normals = primitive.normal().unwrap();
    let indices = primitive.indices().unwrap();

    let vertices: Vec<Vertex> = positions.zip(normals)
    .map(|(position, normal)| Vertex {
        position: Vector3::from(position),
        normal: Vector3::from(normal),
        ..Vertex::default()
    })
    .collect();

    let indices: Vec<u32> = match indices {
        Indices::U8(indices) => indices.map(|i| i as u32).collect(),
        Indices::U16(indices) => indices.map(|i| i as u32).collect(),
        Indices::U32(indices) => indices.map(|i| i as u32).collect(),
    };

    // TODO: No debug
    // assert_eq!(primitive.mode(), Mode::Triangles);
    let textures = Vec::new();
    RenderMesh::new(vertices, indices, textures)
}
