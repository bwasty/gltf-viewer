use std;
use std::io::Read;
use gltf::v2::{ import, Root };
use gltf::v2::accessor::{ Accessor, ComponentType };
use gltf::v2::buffer::{ Target, BufferView};
use gltf::v2::mesh::{ Mode, Primitive };

use gfx;

pub fn load_file(path: &str) {
    let root = import(path);
    match root {
        Ok(root) => {
            println!("glTF version 2.0");
            // println!("{:#?}", root);
            load_box(&root);
            load_box_2(&root);
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
    let normal_buffer_view = &root.buffer_view(normals_accessor.buffer_view.clone());
    let normal_data = &buffer_contents[
        normal_buffer_view.byte_offset as usize .. (normal_buffer_view.byte_offset + normal_buffer_view.byte_length) as usize
    ];

    println!("pos len: {}", position_data.len());
    println!("idx len: {}", index_data.len());
    println!("nml len: {}", normal_data.len());
}

pub fn load_box_2(root: &Root) {
    let buffer_data = root.load_buffer(0);

    let mesh = &root.meshes()[0];
    let primitive = &mesh.primitives[0];
    let position_data = root.get_attribute_data(&buffer_data, primitive, "POSITION");
}

trait DataAccessor<'a> {
    fn load_buffer(&self, index: usize) -> Vec<u8>;
    fn get_buffer_view_data(&self, buffer_data: &'a [u8], buffer_view_index: usize) -> &'a [u8];
    fn get_attribute_data(&self, buffer_data: &'a [u8], primitive: &Primitive, attribute_name: &str) -> &'a [u8];
    // fn get_index_data(&self, primitive: &Primitive);
}

impl <'a> DataAccessor<'a> for Root {
    fn load_buffer(&self, index: usize) -> Vec<u8> {
        let buffer = &self.buffers()[index];
        // TODO!: determine base directory... / handle non-file URIs
        let mut file = std::fs::File::open(format!("src/data/{}", buffer.uri)).unwrap();
        let mut buffer_data = Vec::with_capacity(buffer.byte_length as usize);
        file.read_to_end(&mut buffer_data).unwrap();
        assert_eq!(buffer_data.len(), buffer.byte_length as usize); 
        buffer_data
    }

    fn get_buffer_view_data(&self, buffer_data: &'a [u8], buffer_view_index: usize) -> &'a [u8] {
        let buffer_view = &self.buffer_views()[buffer_view_index];
        // TODO!!!: must take into account buffer_view.buffer!!
         &buffer_data[buffer_view.byte_offset as usize .. (buffer_view.byte_offset + buffer_view.byte_length) as usize]
    }

    fn get_attribute_data(&self, buffer_data: &'a [u8], primitive: &Primitive, attribute_name: &str) -> &'a [u8] {
        // TODO!: handle non-existing attr names
        let accessor_index = primitive.attributes[attribute_name].clone();
        let accessor = &self.accessor(accessor_index);
        self.get_buffer_view_data(buffer_data, accessor.buffer_view.value() as usize)
    }
    
}

// type ColorFormat = gfx::format::Rgba8;
// type DepthFormat = gfx::format::DepthStencil;

// gfx_defines!{
//     #[derive(Default)]
//     vertex BoxVertex {
//         pos: [f32; 4] = "a_Pos",
//         normal: [f32; 3] = "a_Normal",
//     }

//     constant BoxLocals {
//         transform: [[f32; 4]; 4] = "u_Transform",
//     }

//     pipeline BoxPipe {
//         vbuf: gfx::VertexBuffer<BoxVertex> = (),
//         transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
//         locals: gfx::ConstantBuffer<BoxVertex> = "Locals",
//         // color: gfx::TextureSampler<[f32; 4]> = "t_Color",
//         out_color: gfx::RenderTarget<ColorFormat> = "Target0",
//         out_depth: gfx::DepthTarget<DepthFormat> =
//             gfx::preset::depth::LESS_EQUAL_WRITE,
//     }
// }
