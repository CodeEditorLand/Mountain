//! # Performance Metrics Tracking
//!
//! Provides performance measurement and tracking for IPC components.
//! Used to monitor throughput, latency, and resource usage.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Performance metrics for IPC operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Messages per second (throughput)
    pub messages_per_second: f64,
    /// Average response time in milliseconds
    pub average_latency_ms: f64,
    /// Peak response time in milliseconds
    pub peak_latency_ms: f64,
    /// Compression ratio (compressed_size / original_size)
    pub compression_ratio: f64,
    /// Connection pool utilization (0.0 to 1.0)
    pub pool_utilization: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// CPU usage as percentage (0.0 to 100.0)
    pub cpu_usage_percent: f64,
    /// Total messages processed
    pub total_messages: u64,
    /// Failed messages
    pub failed_messages: u64,
    /// Last update timestamp
    pub last_updated: Instant,
}

impl PerformanceMetrics {
    /// Create new performance metrics with default values
    pub fn new() -> Self {
        Self {
            messages_per_second: 0.0,
            average_latency_ms: 0.0,
            peak_latency_ms: 0.0,
            compression_ratio: 1.0,
            pool_utilization: 0.0,
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
            total_messages: 0,
            failed_messages: 0,
            last_updated: Instant::now(),
        }
    }

    /// Record a successful message with its latency
    pub fn record_message(&mut self, latency: Duration) {
        let latency_ms = latency.as_millis() as f64;
        
        // Update average latency
        if self.total_messages > 0 {
            self.average_latency_ms = (self.average_latency_ms * self.total_messages as f64 + latency_ms)
                / (self.total_messages + 1) as f64;
        } else {
            self.average_latency_ms = latency_ms;
        }

        // Update peak latency
        if latency_ms > self.peak_latency_ms {
            self.peak_latency_ms = latency_ms;
        }

        self.total_messages += 1;
        self.last_updated = Instant::now();
    }

    /// Record a failed message
    pub fn record_failure(&mut self) {
        self.failed_messages += 1;
        self.last_updated = Instant::now();
    }

    /// Calculate success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_messages == 0 {
            return 1.0;
        }
        1.0 - (self.failed_messages as f64 / self.total_messages as f64)
    }

    /// Check if latency is within acceptable thresholds
    pub fn is_latency_acceptable(&self, threshold_ms: f64) -> bool {
        self.average_latency_ms <= threshold_ms && self.peak_latency_ms <= threshold_ms * 2.0
    }

    /// Get message success rate as percentage
    pub fn success_rate_percent(&self) -> f64 {
        self.success_rate() * 100.0
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Throughput metrics for measuring message flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Messages received
    pub messages_received: u64,
    /// Messages sent
    pub messages_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Start time of measurement period
    pub start_time: Instant,
}

impl ThroughputMetrics {
    /// Create new throughput metrics
    pub fn new() -> Self {
        Self {
            messages_received: 0,
            messages_sent: 0,
            bytes_received: 0,
            bytes_sent: 0,
            start_time: Instant::now(),
        }
    }

    /// Record a received message
    pub fn record_received(&mut self, bytes: u64) {
        self.messages_received += 1;
        self.bytes_received += bytes;
    }

    /// Record a sent message
    pub fn record_sent(&mut self, bytes: u64) {
        self.messages_sent += 1;
        self.bytes_sent += bytes;
    }

    /// Calculate messages per second received
    pub fn messages_per_second_received(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.messages_received as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Calculate messages per second sent
    pub fn messages_per_second_sent(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.messages_sent as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Calculate bytes per second received
    pub fn bytes_per_second_received(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.bytes_received as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Calculate bytes per second sent
    pub fn bytes_per_second_sent(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.bytes_sent as f64 / elapsed
        } else {
            0.0
        }
    }
}

impl Default for ThroughputMetrics {
    fn default() -> Self {
        Self::new()
    }
}
