use std::rc::Rc;

use render::Primitive;

pub struct Mesh {
    pub primitives: Vec<Rc<Primitive>>,
    // TODO
    // pub weights: Vec<Rc<?>>
    pub name: Option<String>
}
