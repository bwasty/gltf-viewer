use std::time::SystemTime;

#[allow(dead_code)]
pub fn elapsed(start_time: &SystemTime) -> String {
    let elapsed = start_time.elapsed().unwrap();
    let secs = elapsed.as_secs();
    let ms = elapsed.subsec_nanos() as f64 / 1_000_000.0;
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
