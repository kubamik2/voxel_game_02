use std::sync::{atomic::AtomicBool, Arc};

pub struct Interval {
    interval: std::time::Duration,
    last_execution_time: std::time::Instant,
}

impl Interval {
    pub fn new(interval: std::time::Duration) -> Self {
        Self { interval, last_execution_time: std::time::Instant::now() }
    }

    pub fn tick<F: FnOnce()>(&mut self, f: F) {
        if self.last_execution_time.elapsed() >= self.interval {
            (f)();
            self.last_execution_time = std::time::Instant::now();
        }
    }
}

pub struct IntervalThread {
    quit: Arc<AtomicBool>,
}

impl IntervalThread {
    pub fn new<F>(interval: std::time::Duration, f: F) -> Self where F: FnMut() + Send + 'static {
        let quit = Arc::new(AtomicBool::new(false));
        let quit_clone = quit.clone();
        rayon::spawn(move || Self::run_scheduler(interval, f, quit_clone));
        Self { quit }
    }

    fn run_scheduler<F>(interval: std::time::Duration, mut f: F, quit: Arc<AtomicBool>) where F: FnMut() {
        let mut last_execution_duration = std::time::Duration::ZERO;
        loop {
            std::thread::sleep(interval.saturating_sub(last_execution_duration));
            let now = std::time::Instant::now();
            if quit.load(std::sync::atomic::Ordering::Relaxed) { return; }
            (f)();
            last_execution_duration = now.elapsed();
        }
    }

    pub fn quit(self) {
        self.quit.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Drop for IntervalThread {
    fn drop(&mut self) {
        self.quit.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}