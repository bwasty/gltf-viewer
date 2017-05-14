use gltf;


pub fn load_file(path: &str) {
    let root = gltf::v2::import(path);
    match root {
        Ok(root) => {
            println!("glTF version 2.0");
            println!("{:?}", root);
        }
        Err(err) => {
            println!("Error: {:#?}", err);
        }
    }
}
