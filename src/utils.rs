#![macro_use]

use std::time::{Duration, Instant};

use log::{info};

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
