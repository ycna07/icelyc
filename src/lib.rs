use std::thread;
use std::time::{Duration, Instant};

pub struct Timer {
    start_time: Instant,
    paused_duration: Duration,
    is_paused: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            paused_duration: Duration::new(0, 0),
            is_paused: true,
        }
    }

    pub fn pause(&mut self) {
        if !self.is_paused {
            self.paused_duration += Instant::now() - self.start_time;
            self.is_paused = true;
        }
    }

    pub fn resume(&mut self) {
        if self.is_paused {
            self.start_time = Instant::now();
            self.is_paused = false;
        }
    }

    pub fn elapsed(&self) -> Duration {
        if self.is_paused {
            self.paused_duration
        } else {
            self.paused_duration + (Instant::now() - self.start_time)
        }
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.paused_duration = Duration::new(0, 0);
        self.is_paused = true;
    }
}
