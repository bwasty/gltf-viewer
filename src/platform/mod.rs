// wasm build
#[cfg(feature = "use_wasm_bindgen")]
pub mod webgl;
#[cfg(feature = "use_wasm_bindgen")]
mod local {
    #[macro_use]
    pub use crate::platform::webgl::*;
}

// application build
#[cfg(not(feature = "use_wasm_bindgen"))]
pub mod gl;
#[cfg(not(feature = "use_wasm_bindgen"))]
mod local {
    pub use crate::platform::gl::*;
}

// export wrapped module
#[macro_use]
pub use crate::platform::local::*;
