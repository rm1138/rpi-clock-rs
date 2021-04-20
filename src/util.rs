use std::time::{Duration, Instant};

pub fn spin_wait(ms: u64) {
    let now = Instant::now();
    while Instant::now().duration_since(now) < Duration::from_millis(ms) {}
}
