//! # Performance and Operational Metrics
//!
//! This module provides comprehensive metrics collection for the Mountain application.
//! It tracks performance indicators, operational metrics, and health statistics.
//!
//! ## Metrics Categories
//!
//! - **Performance Metrics**: Latency, throughput, operation times
//! - **Operational Metrics**: Connection health, error rates, resource usage
//! - **Business Metrics**: Command usage, extension lifecycle, user interactions
//!
//! ## Usage Example
//!
//! ```rust
//! use Mountain::Telemetry::Metrics;
//!
//! // Record a metric
//! Metrics::record_gauge("ipc.connection_count", 5.0);
//!
//! // Time an operation
//! let timer = Metrics::start_timer("command.execute");
//! // ... do work ...
//! timer.stop_and_record();
//! ```

use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

// ============================================================================
// Metric Types
// ============================================================================

/// A metric value with metadata
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub timestamp: std::time::SystemTime,
    pub labels: HashMap<String, String>,
}

/// Different types of metric values
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// A single numerical value that can go up or down
    Counter(f64),
    /// A single numerical value (gauge)
    Gauge(f64),
    /// A duration measurement
    Histogram(Duration),
    /// A boolean value
    Boolean(bool),
    /// A string value
    Text(String),
}

// ============================================================================
// Metrics Registry
// ============================================================================

/// Central registry for all metrics
#[derive(Debug)]
pub struct MetricsRegistry {
    metrics: Arc<RwLock<Vec<Metric>>>,
    max_entries: usize,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new(max_entries: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::with_capacity(max_entries))),
            max_entries,
        }
    }
    
    /// Record a counter metric
    pub fn record_counter(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: MetricValue::Counter(value),
            timestamp: std::time::SystemTime::now(),
            labels,
        };
        self.push_metric(metric);
    }
    
    /// Record a gauge metric
    pub fn record_gauge(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: MetricValue::Gauge(value),
            timestamp: std::time::SystemTime::now(),
            labels,
        };
        self.push_metric(metric);
    }
    
    /// Record a histogram/duration metric
    pub fn record_histogram(&self, name: &str, value: Duration, labels: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: MetricValue::Histogram(value),
            timestamp: std::time::SystemTime::now(),
            labels,
        };
        self.push_metric(metric);
    }
    
    fn push_metric(&self, metric: Metric) {
        let mut metrics = self.metrics.write();
        if metrics.len() >= self.max_entries {
            metrics.remove(0); // Remove oldest
        }
        metrics.push(metric);
    }
    
    /// Get all metrics
    pub fn get_all_metrics(&self) -> Vec<Metric> {
        self.metrics.read().clone()
    }
    
    /// Get metrics by name
    pub fn get_metrics_by_name(&self, name: &str) -> Vec<Metric> {
        self.metrics.read()
            .iter()
            .filter(|m| m.name == name)
            .cloned()
            .collect()
    }
}

/// Global metrics registry instance
lazy_static::lazy_static! {
    static ref GLOBAL_REGISTRY: Arc<MetricsRegistry> = 
        Arc::new(MetricsRegistry::new(10000));
}

// ============================================================================
// Timer Helper
// ============================================================================

/// A timer for measuring execution time
pub struct Timer {
    name: String,
    labels: HashMap<String, String>,
    start: Instant,
}

impl Timer {
    /// Start a new timer
    pub fn start(name: &str) -> Self {
        Self {
            name: name.to_string(),
            labels: HashMap::new(),
            start: Instant::now(),
        }
    }
    
    /// Add a label to the timer
    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Stop the timer and record the duration
    pub fn stop_and_record(self) -> Duration {
        let duration = self.start.elapsed();
        GLOBAL_REGISTRY.record_histogram(&self.name, duration, self.labels);
        duration
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Record a counter metric
pub fn record_counter(name: &str, value: f64) {
    GLOBAL_REGISTRY.record_counter(name, value, HashMap::new());
}

/// Record a gauge metric
pub fn record_gauge(name: &str, value: f64) {
    GLOBAL_REGISTRY.record_gauge(name, value, HashMap::new());
}

/// Get all metrics
pub fn get_all_metrics() -> Vec<Metric> {
    GLOBAL_REGISTRY.get_all_metrics()
}

// ============================================================================
// Initialization
// ============================================================================

/// Initialize metrics collection
#[cfg(feature = "Telemetry")]
pub fn initialize_metrics() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Metrics system initialized");
    Ok(())
}

/// Initialize metrics (no-op when Telemetry feature is disabled)
#[cfg(not(feature = "Telemetry"))]
pub fn initialize_metrics() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_creation() {
        let registry = MetricsRegistry::new(100);
        let metrics = registry.get_all_metrics();
        assert!(metrics.is_empty());
    }
    
    #[test]
    fn test_counter_recording() {
        let registry = MetricsRegistry::new(100);
        registry.record_counter("test.counter", 42.0, HashMap::new());
        
        let metrics = registry.get_all_metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "test.counter");
    }
    
    #[test]
    fn test_gauge_recording() {
        let registry = MetricsRegistry::new(100);
        registry.record_gauge("test.gauge", 99.9, HashMap::new());
        
        let metrics = registry.get_all_metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "test.gauge");
    }
    
    #[test]
    fn test_timer() {
        let timer = Timer::start("test.timer");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = timer.stop_and_record();
        
        assert!(duration.as_millis() >= 10);
    }
}
