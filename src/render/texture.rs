use gltf;

pub struct Texture {
    pub index: usize, // glTF index
    pub name: Option<String>,

    pub id: u32, // OpenGL id

    // TODO notes
    // sampler - merge here?
    // image -> Rc
}

impl Texture {
    pub fn from_gltf(g_texture: &gltf::texture::Texture) -> Texture {

        // let image = g_texture.source();
        Texture {
            index: g_texture.index(),
            name: g_texture.name().map(|s| s.into()),
            id: 0, // TODO!!!
        }
    }
}
