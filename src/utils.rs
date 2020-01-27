#![macro_use]

use std::ffi::CStr;
use std::mem;
use std::time::{Duration, Instant};

use gl;
use gl::types::GLubyte;

use log::{debug, info, error};

pub fn elapsed(start_time: Instant) -> String {
    let elapsed = start_time.elapsed();
    format_duration(elapsed)
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();
    let ms = f64::from(nanos) / 1_000_000.0;
    if secs > 0 {
        let secs = secs as f64 + ms / 1000.0;
        format!("{:<4.*} s", 1, secs)
    }
    else {
        let places =
            if ms >= 20.0      { 0 }
            else if ms >= 1.0  { 1 }
            else {
                let micros = f64::from(nanos) / 1000.0;
                let places = if micros >= 10.0 { 0 } else { 2 };
                return format!("{:>3.*} µs", places, micros)
            };
        format!("{:>3.*} ms", places, ms)
    }
}

pub fn print_elapsed(message: &str, start_time: Instant) {
    info!("{:<25}{}", message, elapsed(start_time));
}

pub struct FrameTimer {
    message: String,
    averaging_window: usize,
    current_frame_start: Instant,
    pub frame_times: Vec<Duration>,
}

/// Timing helper that averages timings over `averaging_window`
// frames and then prints avg/min/max
impl FrameTimer {
    pub fn new(message: &str, averaging_window: usize) -> FrameTimer {
        FrameTimer {
            message: message.to_owned(),
            averaging_window,
            current_frame_start: Instant::now(),
            frame_times: Vec::with_capacity(averaging_window),
        }
    }

    pub fn start(&mut self) {
        self.current_frame_start = Instant::now();
    }

    pub fn end(&mut self) {
        self.frame_times.push(self.current_frame_start.elapsed());
        if self.frame_times.len() == self.averaging_window {
            self.print_and_reset();
        }
    }

    pub fn print_and_reset(&mut self) {
        {
            let avg = self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32;
            let min = self.frame_times.iter().min().unwrap();
            let max = self.frame_times.iter().max().unwrap();
            info!("{:<15}{} (min: {}, max: {})", self.message,
                format_duration(avg), format_duration(*min), format_duration(*max));
        }
        self.frame_times.clear();
    }
}

pub unsafe fn gl_check_error(file: &str, line: u32) -> u32 {
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

        error!("{} | {} ({})", error, file, line);

        error_code = gl::GetError();
    }
    error_code
}

#[allow(unused_macros)]
macro_rules! gl_check_error {
    () => (
        gl_check_error(file!(), line!())
    )
}

/// Prints information about the current OpenGL context
/// Based on glium's `context::capabilities::get_capabilities`
pub unsafe fn print_context_info() {
    debug!("Renderer     : {}", gl_string(gl::GetString(gl::RENDERER)));
    debug!("Vendor       : {}", gl_string(gl::GetString(gl::VENDOR)));
    debug!("Version      : {}", gl_string(gl::GetString(gl::VERSION)));
    debug!("GLSL         : {}", gl_string(gl::GetString(gl::SHADING_LANGUAGE_VERSION)));

    let mut val = mem::MaybeUninit::uninit().assume_init();
    gl::GetIntegerv(gl::CONTEXT_PROFILE_MASK, &mut val);
    let val = val as gl::types::GLenum;
    let profile = if (val & gl::CONTEXT_COMPATIBILITY_PROFILE_BIT) != 0 {
        "Compatibility"
    } else if (val & gl::CONTEXT_CORE_PROFILE_BIT) != 0 {
        "Core"
    } else {
        "None"
    };
    debug!("Profile      : {}", profile);

    let (debug, forward_compatible) = {
        let mut val = mem::MaybeUninit::uninit().assume_init();
        gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut val);
        let val = val as gl::types::GLenum;
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
