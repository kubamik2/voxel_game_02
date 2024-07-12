pub struct Interval<F> where F: FnMut() {
    function: F,
    interval: std::time::Duration,
    last_execution_time: std::time::Instant,
}

impl<F> Interval<F> where F: FnMut() {
    pub fn new(interval: std::time::Duration, f: F) -> Self {
        Self { function: f, interval, last_execution_time: std::time::Instant::now() }
    }

    pub fn tick(&mut self) {
        if self.last_execution_time.elapsed() >= self.interval {
            (self.function)();
            self.last_execution_time = std::time::Instant::now();
        }
    }
}

pub struct IntervalThread<F> where F: FnMut() + Send + 'static + Clone {
    function: F,
    interval: std::time::Duration,
    last_execution_time: std::time::Instant,
}

impl<F> IntervalThread<F> where F: FnMut() + Send + 'static + Clone {
    pub fn new(interval: std::time::Duration, f: F) -> Self {
        Self { function: f, interval, last_execution_time: std::time::Instant::now() }
    }

    pub fn tick(&mut self) {
        if self.last_execution_time.elapsed() >= self.interval {
            rayon::spawn(self.function.clone());
            self.last_execution_time = std::time::Instant::now();
        }
    }
}