use std::time::{Duration, Instant};

/// Session statistics tracked during reading.
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub words_read: usize,
    pub start_time: Option<Instant>,
    pub elapsed_reading: Duration,
    pub current_wpm: u32,
}

impl SessionStats {
    pub fn new(wpm: u32) -> Self {
        Self {
            words_read: 0,
            start_time: None,
            elapsed_reading: Duration::ZERO,
            current_wpm: wpm,
        }
    }

    /// Average WPM based on actual words read and elapsed time.
    pub fn average_wpm(&self) -> f64 {
        let elapsed_minutes = self.total_elapsed().as_secs_f64() / 60.0;
        if elapsed_minutes <= 0.0 {
            return 0.0;
        }
        self.words_read as f64 / elapsed_minutes
    }

    /// Record the start of a reading session (or resume).
    pub fn start_reading(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Pause reading: accumulate elapsed time since start_time and clear it.
    pub fn pause_reading(&mut self) {
        if let Some(start) = self.start_time.take() {
            self.elapsed_reading += start.elapsed();
        }
    }

    /// Record that a word has been read.
    pub fn record_word(&mut self) {
        self.words_read += 1;
    }

    /// Total elapsed reading time including any current active session.
    fn total_elapsed(&self) -> Duration {
        let active = self
            .start_time
            .map(|s| s.elapsed())
            .unwrap_or(Duration::ZERO);
        self.elapsed_reading + active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn new_stats() {
        let stats = SessionStats::new(300);
        assert_eq!(stats.words_read, 0);
        assert!(stats.start_time.is_none());
        assert_eq!(stats.elapsed_reading, Duration::ZERO);
        assert_eq!(stats.current_wpm, 300);
    }

    #[test]
    fn average_wpm_no_elapsed_is_zero() {
        let stats = SessionStats::new(300);
        assert_eq!(stats.average_wpm(), 0.0);
    }

    #[test]
    fn record_word_increments() {
        let mut stats = SessionStats::new(300);
        stats.record_word();
        stats.record_word();
        stats.record_word();
        assert_eq!(stats.words_read, 3);
    }

    #[test]
    fn start_and_pause_accumulates_elapsed() {
        let mut stats = SessionStats::new(300);
        stats.start_reading();
        thread::sleep(Duration::from_millis(50));
        stats.pause_reading();
        assert!(stats.elapsed_reading >= Duration::from_millis(30));
        assert!(stats.start_time.is_none());
    }

    #[test]
    fn average_wpm_calculation() {
        let mut stats = SessionStats::new(300);
        // Simulate reading 10 words in roughly 1 second
        stats.words_read = 10;
        stats.elapsed_reading = Duration::from_secs(60); // exactly 1 minute
        // 10 words / 1 minute = 10 WPM
        let avg = stats.average_wpm();
        assert!((avg - 10.0).abs() < 0.01);
    }

    #[test]
    fn pause_without_start_is_noop() {
        let mut stats = SessionStats::new(300);
        stats.pause_reading(); // should not panic
        assert_eq!(stats.elapsed_reading, Duration::ZERO);
    }

    #[test]
    fn multiple_start_pause_cycles() {
        let mut stats = SessionStats::new(300);
        stats.start_reading();
        thread::sleep(Duration::from_millis(30));
        stats.pause_reading();
        let first = stats.elapsed_reading;

        stats.start_reading();
        thread::sleep(Duration::from_millis(30));
        stats.pause_reading();
        assert!(stats.elapsed_reading > first);
    }
}
