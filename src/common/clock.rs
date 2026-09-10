//! High-performance clock (same design as sol-parser-sdk for consistent grpc_recv_us vs "now").
//!
//! Uses monotonic clock + base UTC timestamp to avoid frequent syscalls; aligned with sol-parser-sdk
//! so event-side grpc_recv_us and SDK-side now_micros() share the same time scale.

use std::time::Instant;

/// High-performance clock: monotonic + base UTC microsecond timestamp.
#[derive(Debug)]
pub struct HighPerformanceClock {
    base_instant: Instant,
    base_timestamp_us: i64,
    last_calibration: Instant,
    calibration_interval_secs: u64,
}

impl HighPerformanceClock {
    /// Calibrate every 5 minutes by default.
    pub fn new() -> Self {
        Self::new_with_calibration_interval(300)
    }

    /// Sample multiple times and use the lowest-latency baseline to reduce init error.
    pub fn new_with_calibration_interval(calibration_interval_secs: u64) -> Self {
        let mut best_offset = i64::MAX;
        let mut best_instant = Instant::now();
        let mut best_timestamp = chrono::Utc::now().timestamp_micros();

        for _ in 0..3 {
            let instant_before = Instant::now();
            let timestamp = chrono::Utc::now().timestamp_micros();
            let instant_after = Instant::now();
            let sample_latency = instant_after.duration_since(instant_before).as_nanos() as i64;
            if sample_latency < best_offset {
                best_offset = sample_latency;
                best_instant = instant_before;
                best_timestamp = timestamp;
            }
        }

        Self {
            base_instant: best_instant,
            base_timestamp_us: best_timestamp,
            last_calibration: best_instant,
            calibration_interval_secs,
        }
    }

    #[inline(always)]
    pub fn now_micros(&self) -> i64 {
        let elapsed = self.base_instant.elapsed();
        self.base_timestamp_us + elapsed.as_micros() as i64
    }

    /// Recalibrate when needed to prevent drift.
    pub fn now_micros_with_calibration(&mut self) -> i64 {
        if self.last_calibration.elapsed().as_secs() >= self.calibration_interval_secs {
            self.recalibrate();
        }
        self.now_micros()
    }

    fn recalibrate(&mut self) {
        let current_monotonic = Instant::now();
        let current_utc = chrono::Utc::now().timestamp_micros();
        let expected_utc = self.base_timestamp_us
            + current_monotonic.duration_since(self.base_instant).as_micros() as i64;
        let drift_us = current_utc - expected_utc;
        if drift_us.abs() > 1000 {
            self.base_instant = current_monotonic;
            self.base_timestamp_us = current_utc;
        }
        self.last_calibration = current_monotonic;
    }
}

impl Default for HighPerformanceClock {
    fn default() -> Self {
        Self::new()
    }
}

static HIGH_PERF_CLOCK: once_cell::sync::OnceCell<HighPerformanceClock> =
    once_cell::sync::OnceCell::new();

/// Current time in microseconds (UTC scale); same as sol-parser-sdk clock::now_micros for comparable grpc_recv_us.
#[inline(always)]
pub fn now_micros() -> i64 {
    let clock = HIGH_PERF_CLOCK.get_or_init(HighPerformanceClock::new);
    clock.now_micros()
}

/// Elapsed microseconds from start_timestamp_us to now.
#[inline(always)]
pub fn elapsed_micros_since(start_timestamp_us: i64) -> i64 {
    now_micros() - start_timestamp_us
}
