use std::time::{SystemTime, Duration};

#[allow(dead_code)]
pub fn elapsed(start_time: &SystemTime) -> String {
    let elapsed = start_time.elapsed().unwrap();
    format_duration(elapsed)
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let ms = duration.subsec_nanos() as f64 / 1_000_000.0;
    if secs > 0 {
        let secs = secs as f64 + ms / 1000.0;
        format!("{:.*}s", 1, secs)
    }
    else {
        let places =
            if ms > 20.0      { 0 }
            else if ms > 1.0  { 1 }
            else              { 3 };
        format!("{:.*}ms", places, ms)
    }
}

pub fn print_elapsed(message: &str, start_time: &SystemTime) {
    println!("{:<20}{}", message, elapsed(start_time));
}

pub struct FrameTimer {
    message: String,
    averaging_window: usize,
    current_frame_start: SystemTime,
    frame_times: Vec<Duration>,
}

/// Timing helper that averages timings over `averaging_window`
// frames and then prints avg/min/max
impl FrameTimer {
    pub fn new(message: &str, averaging_window: usize) -> FrameTimer {
        FrameTimer {
            message: message.to_owned(),
            averaging_window: averaging_window,
            current_frame_start: SystemTime::now(),
            frame_times: Vec::with_capacity(averaging_window),
        }
    }

    pub fn start(&mut self) {
        self.current_frame_start = SystemTime::now();
    }

    pub fn end(&mut self) {
        if let Ok(elapsed) = self.current_frame_start.elapsed() {
            self.frame_times.push(elapsed);
            if self.frame_times.len() == self.averaging_window {
                self.print_and_reset();
            }
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
