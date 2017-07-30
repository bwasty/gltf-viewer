#![macro_use]

use std::time::{Duration, Instant};

use gl;

pub fn elapsed(start_time: &Instant) -> String {
    let elapsed = start_time.elapsed();
    format_duration(elapsed)
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();
    let ms =  nanos as f64 / 1_000_000.0;
    if secs > 0 {
        let secs = secs as f64 + ms / 1000.0;
        format!("{:<4.*} s", 1, secs)
    }
    else {
        let places =
            if ms >= 20.0      { 0 }
            else if ms >= 1.0  { 1 }
            else {
                let micros = nanos as f64 / 1000.0;
                let places = if micros >= 10.0 { 0 } else { 2 };
                return format!("{:>3.*} µs", places, micros)
            };
        format!("{:>3.*} ms", places, ms)
    }
}

pub fn print_elapsed(message: &str, start_time: &Instant) {
    println!("{:<25}{}", message, elapsed(start_time));
}

pub struct FrameTimer {
    message: String,
    averaging_window: usize,
    current_frame_start: Instant,
    frame_times: Vec<Duration>,
}

/// Timing helper that averages timings over `averaging_window`
// frames and then prints avg/min/max
impl FrameTimer {
    pub fn new(message: &str, averaging_window: usize) -> FrameTimer {
        FrameTimer {
            message: message.to_owned(),
            averaging_window: averaging_window,
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
            println!("{:<15}{} (min: {}, max: {})", self.message,
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

        println!("{} | {} ({})", error, file, line);

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

/// Determine if a number is a power of two, for texture resizing
/// Source: https://graphics.stanford.edu/~seander/bithacks.html#DetermineIfPowerOf2
/// Using the simpler version that considers 0 a power of two - irrelevant here.
pub fn is_power_of_two(v: u32) -> bool {
    (v & (v - 1)) == 0
}

/// Determine the next highest power of two, for texture resizing
/// Source: https://graphics.stanford.edu/~seander/bithacks.html#RoundUpPowerOf2
pub fn next_power_of_two(mut v: u32) -> u32 {
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v + 1
}
