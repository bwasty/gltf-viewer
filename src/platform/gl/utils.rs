#![macro_use]

use std::ffi::CStr;

use gl;
use gl::types::GLubyte;

use log::{debug, error};

pub unsafe fn gl_check_error() -> u32 {
    let mut error_code = gl::GetError();
    while error_code != gl::NO_ERROR {
        let error = match error_code {
            gl::INVALID_ENUM => "INVALID_ENUM",
            gl::INVALID_VALUE => "INVALID_VALUE",
            gl::INVALID_OPERATION => "INVALID_OPERATION",
            gl::STACK_OVERFLOW => "STACK_OVERFLOW",
            gl::STACK_UNDERFLOW => "STACK_UNDERFLOW",
            gl::OUT_OF_MEMORY => "OUT_OF_MEMORY",
            gl::INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
            _ => "unknown GL error code"
        };

        error!("{} | {} ({})", error, file!(), line!());

        error_code = gl::GetError();
    }
    error_code
}


/// Prints information about the current OpenGL context
/// Based on glium's `context::capabilities::get_capabilities`
pub unsafe fn print_context_info() {
    debug!("Renderer     : {}", gl_string(gl::GetString(gl::RENDERER)));
    debug!("Vendor       : {}", gl_string(gl::GetString(gl::VENDOR)));
    debug!("Version      : {}", gl_string(gl::GetString(gl::VERSION)));
    debug!("GLSL         : {}", gl_string(gl::GetString(gl::SHADING_LANGUAGE_VERSION)));

    let mut val = core::mem::MaybeUninit::<i32>::uninit();
    gl::GetIntegerv(gl::CONTEXT_PROFILE_MASK, val.as_mut_ptr());
    let val = val.assume_init() as gl::types::GLenum;
    let profile = if (val & gl::CONTEXT_COMPATIBILITY_PROFILE_BIT) != 0 {
        "Compatibility"
    } else if (val & gl::CONTEXT_CORE_PROFILE_BIT) != 0 {
        "Core"
    } else {
        "None"
    };
    debug!("Profile      : {}", profile);

    let (debug, forward_compatible) = {
        let mut val = core::mem::MaybeUninit::<i32>::uninit();
        gl::GetIntegerv(gl::CONTEXT_FLAGS, val.as_mut_ptr());
        let val = val.assume_init() as gl::types::GLenum;
        ((val & gl::CONTEXT_FLAG_DEBUG_BIT) != 0,
         (val & gl::CONTEXT_FLAG_FORWARD_COMPATIBLE_BIT) != 0)
    };
    debug!("Context Flags: Debug: {}, Forward Compatible: {}", debug, forward_compatible);

    let mut num_extensions = 0;
    gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut num_extensions);
    let extensions: Vec<_> = (0 .. num_extensions).map(|num| {
        gl_string(gl::GetStringi(gl::EXTENSIONS, num as gl::types::GLuint))
    }).collect();
    debug!("Extensions   : {}", extensions.join(", "))
}

pub unsafe fn gl_string(raw_string: *const GLubyte) -> String {
    if raw_string.is_null() { return "(NULL)".into() }
    String::from_utf8(CStr::from_ptr(raw_string as *const _).to_bytes().to_vec())
                                .expect("gl_string: non-UTF8 string")
}
