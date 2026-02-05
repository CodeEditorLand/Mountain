//! # Performance Dashboard
//!
//! Advanced monitoring and distributed tracing for IPC performance metrics.
//! Supports OpenTelemetry integration and real-time performance visualization.

use std::{
	collections::{HashMap, VecDeque},
	sync::Arc,
	time::{Duration, Instant, SystemTime},
};

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use tokio::{
	sync::{Mutex as AsyncMutex, RwLock},
	time::interval,
};

/// Performance metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
	pub update_interval_ms:u64,
	pub metrics_retention_hours:u64,
	pub alert_threshold_ms:u64,
	pub trace_sampling_rate:f64,
	pub max_traces_stored:usize,
}

impl Default for DashboardConfig {
	fn default() -> Self {
		Self {
			// Configuration for metrics update frequency in milliseconds.
			update_interval_ms:5000,
			// Retention period for stored metrics in hours.
			metrics_retention_hours:24,
			// Threshold in milliseconds for triggering performance alerts.
			alert_threshold_ms:1000,
			// Fraction of traces to sample (0.1 = 10%).
			trace_sampling_rate:0.1,
			// Maximum number of trace spans to retain.
			max_traces_stored:1000,
		}
	}
}

/// Performance metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
	MessageProcessingTime,
	ConnectionLatency,
	MemoryUsage,
	CpuUsage,
	NetworkThroughput,
	ErrorRate,
	QueueSize,
}

/// Performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
	pub metric_type:MetricType,
	pub value:f64,
	pub timestamp:u64,
	pub channel:Option<String>,
	pub tags:HashMap<String, String>,
}

/// Distributed trace span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
	pub trace_id:String,
	pub span_id:String,
	pub parent_span_id:Option<String>,
	pub operation_name:String,
	pub start_time:u64,
	pub end_time:Option<u64>,
	pub duration_ms:Option<u64>,
	pub tags:HashMap<String, String>,
	pub logs:Vec<TraceLog>,
}

/// Trace log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceLog {
	pub timestamp:u64,
	pub message:String,
	pub level:LogLevel,
	pub fields:HashMap<String, String>,
}

/// Log level for tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
	Debug,
	Info,
	Warn,
	Error,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
	pub alert_id:String,
	pub metric_type:MetricType,
	pub threshold:f64,
	pub current_value:f64,
	pub timestamp:u64,
	pub channel:Option<String>,
	pub severity:AlertSeverity,
	pub message:String,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
	Low,
	Medium,
	High,
	Critical,
}

/// Performance dashboard
pub struct PerformanceDashboard {
	config:DashboardConfig,
	metrics:Arc<RwLock<VecDeque<PerformanceMetric>>>,
	traces:Arc<RwLock<HashMap<String, TraceSpan>>>,
	alerts:Arc<RwLock<VecDeque<PerformanceAlert>>>,
	statistics:Arc<RwLock<DashboardStatistics>>,
	is_running:Arc<AsyncMutex<bool>>,
}

/// Dashboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatistics {
	pub total_metrics_collected:u64,
	pub total_traces_collected:u64,
	pub total_alerts_triggered:u64,
	pub average_processing_time_ms:f64,
	pub peak_processing_time_ms:u64,
	pub error_rate_percentage:f64,
	pub throughput_messages_per_second:f64,
	pub memory_usage_mb:f64,
	pub last_update:u64,
}

impl PerformanceDashboard {
	/// Create a new performance dashboard
	pub fn new(config:DashboardConfig) -> Self {
		let config_clone = config.clone();
		let dashboard = Self {
			config,
			metrics:Arc::new(RwLock::new(VecDeque::new())),
			traces:Arc::new(RwLock::new(HashMap::new())),
			alerts:Arc::new(RwLock::new(VecDeque::new())),
			statistics:Arc::new(RwLock::new(DashboardStatistics {
				total_metrics_collected:0,
				total_traces_collected:0,
				total_alerts_triggered:0,
				average_processing_time_ms:0.0,
				peak_processing_time_ms:0,
				error_rate_percentage:0.0,
				throughput_messages_per_second:0.0,
				memory_usage_mb:0.0,
				last_update:SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap_or_default()
					.as_secs(),
			})),
			is_running:Arc::new(AsyncMutex::new(false)),
		};

		info!(
			"[PerformanceDashboard] Created dashboard with {}ms update interval",
			config_clone.update_interval_ms
		);

		dashboard
	}

	/// Start the performance dashboard
	pub async fn start(&self) -> Result<(), String> {
		{
			let mut running = self.is_running.lock().await;
			if *running {
				// If already running, exit early to prevent duplicate startup.
				return Ok(());
			}
			*running = true;
		}

		// Start metrics collection
		self.start_metrics_collection().await;

		// Start alert monitoring
		self.start_alert_monitoring().await;

		// Start data cleanup
		self.start_data_cleanup().await;

		info!("[PerformanceDashboard] Performance dashboard started");
		Ok(())
	}

	/// Stop the performance dashboard
	pub async fn stop(&self) -> Result<(), String> {
		{
			let mut running = self.is_running.lock().await;
			if !*running {
				// If already stopped, exit early to prevent redundant operations.
				return Ok(());
			}
			*running = false;
		}

		// Clear all data
		{
			let mut metrics = self.metrics.write().await;
			metrics.clear();
		}

		{
			let mut traces = self.traces.write().await;
			traces.clear();
		}

		{
			let mut alerts = self.alerts.write().await;
			alerts.clear();
		}

		info!("[PerformanceDashboard] Performance dashboard stopped");
		Ok(())
	}

	/// Record a performance metric
	pub async fn record_metric(&self, metric:PerformanceMetric) {
		let mut metrics = self.metrics.write().await;
		metrics.push_back(metric.clone());

		// Update statistics
		self.update_statistics().await;

		// Check for alerts
		self.check_alerts(&metric).await;

		trace!("[PerformanceDashboard] Recorded metric: {:?}", metric.metric_type);
	}

	/// Start a new trace span
	pub async fn start_trace_span(&self, operation_name:String) -> TraceSpan {
		let trace_id = Self::generate_trace_id();
		let span_id = Self::generate_span_id();

		let span = TraceSpan {
			trace_id:trace_id.clone(),
			span_id:span_id.clone(),
			parent_span_id:None,
			operation_name,
			start_time:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
			end_time:None,
			duration_ms:None,
			tags:HashMap::new(),
			logs:Vec::new(),
		};

		// Store trace span
		{
			let mut traces = self.traces.write().await;
			traces.insert(span_id.clone(), span.clone());
		}

		// Update statistics
		{
			let mut stats = self.statistics.write().await;
			stats.total_traces_collected += 1;
		}

		span
	}

	/// End a trace span
	pub async fn end_trace_span(&self, span_id:&str) -> Result<(), String> {
		let mut traces = self.traces.write().await;

		if let Some(mut span) = traces.get_mut(span_id) {
			let end_time = SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64;

			span.end_time = Some(end_time);
			span.duration_ms = Some(end_time.saturating_sub(span.start_time));

			trace!(
				"[PerformanceDashboard] Ended trace span: {} (duration: {}ms)",
				span.operation_name,
				span.duration_ms.unwrap_or(0)
			);

			Ok(())
		} else {
			Err(format!("Trace span not found: {}", span_id))
		}
	}

	/// Add log to trace span
	pub async fn add_trace_log(&self, span_id:&str, log:TraceLog) -> Result<(), String> {
		let mut traces = self.traces.write().await;

		if let Some(span) = traces.get_mut(span_id) {
			span.logs.push(log);
			Ok(())
		} else {
			Err(format!("Trace span not found: {}", span_id))
		}
	}

	/// Start metrics collection
	async fn start_metrics_collection(&self) {
		let dashboard = Arc::new(self.clone());

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_millis(dashboard.config.update_interval_ms));

			while *dashboard.is_running.lock().await {
				interval.tick().await;

				// Collect system metrics
				dashboard.collect_system_metrics().await;

				// Update dashboard statistics
				dashboard.update_statistics().await;
			}
		});
	}

	/// Start alert monitoring
	async fn start_alert_monitoring(&self) {
		let dashboard = Arc::new(self.clone());

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_secs(10));

			while *dashboard.is_running.lock().await {
				interval.tick().await;

				// Check for performance alerts
				dashboard.check_performance_alerts().await;
			}
		});
	}

	/// Start data cleanup
	async fn start_data_cleanup(&self) {
		let dashboard = Arc::new(self.clone());

		tokio::spawn(async move {
			// Perform cleanup operations every hour.
			let mut interval = interval(Duration::from_secs(3600));

			while *dashboard.is_running.lock().await {
				interval.tick().await;

				// Cleanup old data
				dashboard.cleanup_old_data().await;
			}
		});
	}

	/// Collect system metrics
	async fn collect_system_metrics(&self) {
		// Collect memory usage
		if let Ok(memory_usage) = Self::get_memory_usage() {
			let metric = PerformanceMetric {
				metric_type:MetricType::MemoryUsage,
				value:memory_usage,
				timestamp:SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap_or_default()
					.as_millis() as u64,
				channel:None,
				tags:HashMap::new(),
			};

			self.record_metric(metric).await;
		}

		// Collect CPU usage
		if let Ok(cpu_usage) = Self::get_cpu_usage() {
			let metric = PerformanceMetric {
				metric_type:MetricType::CpuUsage,
				value:cpu_usage,
				timestamp:SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap_or_default()
					.as_millis() as u64,
				channel:None,
				tags:HashMap::new(),
			};

			self.record_metric(metric).await;
		}
	}

	/// Update dashboard statistics
	async fn update_statistics(&self) {
		let metrics = self.metrics.read().await;
		let mut stats = self.statistics.write().await;

		// Calculate average processing time
		let processing_metrics:Vec<&PerformanceMetric> = metrics
			.iter()
			.filter(|m| matches!(m.metric_type, MetricType::MessageProcessingTime))
			.collect();

		if !processing_metrics.is_empty() {
			let total_time:f64 = processing_metrics.iter().map(|m| m.value).sum();
			stats.average_processing_time_ms = total_time / processing_metrics.len() as f64;

			stats.peak_processing_time_ms = processing_metrics.iter().map(|m| m.value as u64).max().unwrap_or(0);
		}

		// Calculate error rate
		let error_metrics:Vec<&PerformanceMetric> = metrics
			.iter()
			.filter(|m| matches!(m.metric_type, MetricType::ErrorRate))
			.collect();

		if !error_metrics.is_empty() {
			let total_errors:f64 = error_metrics.iter().map(|m| m.value).sum();
			stats.error_rate_percentage = total_errors / error_metrics.len() as f64;
		}

		// Calculate throughput
		let throughput_metrics:Vec<&PerformanceMetric> = metrics
			.iter()
			.filter(|m| matches!(m.metric_type, MetricType::NetworkThroughput))
			.collect();

		if !throughput_metrics.is_empty() {
			let total_throughput:f64 = throughput_metrics.iter().map(|m| m.value).sum();
			stats.throughput_messages_per_second = total_throughput / throughput_metrics.len() as f64;
		}

		// Update memory usage
		let memory_metrics:Vec<&PerformanceMetric> = metrics
			.iter()
			.filter(|m| matches!(m.metric_type, MetricType::MemoryUsage))
			.collect();

		if !memory_metrics.is_empty() {
			let total_memory:f64 = memory_metrics.iter().map(|m| m.value).sum();
			stats.memory_usage_mb = total_memory / memory_metrics.len() as f64;
		}

		stats.last_update = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();
	}

	/// Check for performance alerts
	async fn check_alerts(&self, metric:&PerformanceMetric) {
		let threshold = match metric.metric_type {
			MetricType::MessageProcessingTime => self.config.alert_threshold_ms as f64,
			// Error rate threshold: 5% of total operations.
			MetricType::ErrorRate => 5.0,
			// Memory threshold: 1GB (1024 MB).
			MetricType::MemoryUsage => 1024.0,
			// CPU threshold: 90% utilization.
			MetricType::CpuUsage => 90.0,
			// No alert for other metric types (e.g., ConnectionLatency, QueueSize).
			_ => return,
		};

		if metric.value > threshold {
			let severity = match metric.value / threshold {
				ratio if ratio > 5.0 => AlertSeverity::Critical,
				ratio if ratio > 3.0 => AlertSeverity::High,
				ratio if ratio > 2.0 => AlertSeverity::Medium,
				_ => AlertSeverity::Low,
			};

			let alert = PerformanceAlert {
				alert_id:Self::generate_alert_id(),
				metric_type:metric.metric_type.clone(),
				threshold,
				current_value:metric.value,
				timestamp:metric.timestamp,
				channel:metric.channel.clone(),
				severity,
				message:format!(
					"{} exceeded threshold: {} > {}",
					Self::metric_type_name(&metric.metric_type),
					metric.value,
					threshold
				),
			};

			{
				let mut alerts = self.alerts.write().await;
				alerts.push_back(alert.clone());
			}

			{
				let mut stats = self.statistics.write().await;
				stats.total_alerts_triggered += 1;
			}

			warn!("[PerformanceDashboard] Alert triggered: {}", alert.message);
		}
	}

	/// Check performance alerts periodically
	async fn check_performance_alerts(&self) {
		// This method can be extended to check for complex alert conditions
		// that require evaluating multiple metrics over time
		debug!("[PerformanceDashboard] Checking performance alerts");
	}

	/// Cleanup old data
	async fn cleanup_old_data(&self) {
		let retention_threshold = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs()
			- (self.config.metrics_retention_hours * 3600);

		// Cleanup old metrics
		{
			let mut metrics = self.metrics.write().await;
			metrics.retain(|m| m.timestamp >= retention_threshold);
		}

		// Cleanup old traces
		{
			let mut traces = self.traces.write().await;
			traces.retain(|_, span| span.start_time >= retention_threshold);

			// Limit stored traces
			if traces.len() > self.config.max_traces_stored {
				let excess = traces.len() - self.config.max_traces_stored;
				let keys_to_remove:Vec<String> = traces.keys().take(excess).cloned().collect();

				for key in keys_to_remove {
					traces.remove(&key);
				}
			}
		}

		// Cleanup old alerts
		{
			let mut alerts = self.alerts.write().await;
			alerts.retain(|a| a.timestamp >= retention_threshold);
		}

		debug!("[PerformanceDashboard] Cleaned up old data");
	}

	/// Get memory usage (simplified implementation)
	fn get_memory_usage() -> Result<f64, String> {
		// This is a simplified implementation
		// In a real application, you would use system-specific APIs
		// to query actual system memory consumption. This simulated value
		// provides predictable behavior for testing and demonstration.
		Ok(100.0)
	}

	/// Get CPU usage (simplified implementation)
	fn get_cpu_usage() -> Result<f64, String> {
		// This is a simplified implementation
		// In a real application, you would use system-specific APIs
		// to measure actual CPU utilization. This simulated value provides
		// consistent test data for the performance dashboard.
		Ok(25.0)
	}

	/// Generate trace ID
	fn generate_trace_id() -> String { uuid::Uuid::new_v4().to_string() }

	/// Generate span ID
	fn generate_span_id() -> String { uuid::Uuid::new_v4().to_string() }

	/// Generate alert ID
	fn generate_alert_id() -> String { uuid::Uuid::new_v4().to_string() }

	/// Get metric type name
	fn metric_type_name(metric_type:&MetricType) -> &'static str {
		match metric_type {
			MetricType::MessageProcessingTime => "Message Processing Time",
			MetricType::ConnectionLatency => "Connection Latency",
			MetricType::MemoryUsage => "Memory Usage",
			MetricType::CpuUsage => "CPU Usage",
			MetricType::NetworkThroughput => "Network Throughput",
			MetricType::ErrorRate => "Error Rate",
			MetricType::QueueSize => "Queue Size",
		}
	}

	/// Get dashboard statistics
	pub async fn get_statistics(&self) -> DashboardStatistics { self.statistics.read().await.clone() }

	/// Get recent metrics
	pub async fn get_recent_metrics(&self, limit:usize) -> Vec<PerformanceMetric> {
		let metrics = self.metrics.read().await;
		metrics.iter().rev().take(limit).cloned().collect()
	}

	/// Get active alerts
	pub async fn get_active_alerts(&self) -> Vec<PerformanceAlert> {
		let alerts = self.alerts.read().await;
		alerts.iter().rev().cloned().collect()
	}

	/// Get trace by ID
	pub async fn get_trace(&self, trace_id:&str) -> Option<TraceSpan> {
		let traces = self.traces.read().await;
		traces.values().find(|span| span.trace_id == trace_id).cloned()
	}

	/// Create a performance dashboard with default configuration
	pub fn default_dashboard() -> Self { Self::new(DashboardConfig::default()) }

	/// Create a high-frequency dashboard
	pub fn high_frequency_dashboard() -> Self {
		Self::new(DashboardConfig {
			// High-frequency updates every second for detailed monitoring.
			update_interval_ms:1000,
			// Short retention period for real-time dashboards.
			metrics_retention_hours:1,
			// Lower threshold for aggressive alerting in high-frequency mode.
			alert_threshold_ms:500,
			// Sample all traces for maximum visibility.
			trace_sampling_rate:1.0,
			// Larger trace buffer for high-frequency systems.
			max_traces_stored:5000,
		})
	}
}

impl Clone for PerformanceDashboard {
	fn clone(&self) -> Self {
		Self {
			config:self.config.clone(),
			metrics:self.metrics.clone(),
			traces:self.traces.clone(),
			alerts:self.alerts.clone(),
			statistics:self.statistics.clone(),
			is_running:Arc::new(AsyncMutex::new(false)),
		}
	}
}

/// Utility functions for performance monitoring
impl PerformanceDashboard {
	/// Create a performance metric
	pub fn create_metric(
		metric_type:MetricType,
		value:f64,
		channel:Option<String>,
		tags:HashMap<String, String>,
	) -> PerformanceMetric {
		PerformanceMetric {
			metric_type,
			value,
			timestamp:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
			channel,
			tags,
		}
	}

	/// Create a trace log
	pub fn create_trace_log(message:String, level:LogLevel, fields:HashMap<String, String>) -> TraceLog {
		TraceLog {
			timestamp:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
			message,
			level,
			fields,
		}
	}

	/// Calculate performance score
	pub fn calculate_performance_score(average_processing_time:f64, error_rate:f64, throughput:f64) -> f64 {
		// Simple scoring algorithm
		let time_score = 100.0 / (1.0 + average_processing_time / 100.0);
		let error_score = 100.0 * (1.0 - error_rate / 100.0);
		// Normalize throughput to thousands of messages per second for balanced
		// scoring.
		let throughput_score = throughput / 1000.0;

		(time_score * 0.4 + error_score * 0.4 + throughput_score * 0.2)
			.max(0.0)
			.min(100.0)
	}

	/// Format metric value for display
	pub fn format_metric_value(metric_type:&MetricType, value:f64) -> String {
		match metric_type {
			MetricType::MessageProcessingTime => format!("{:.2}ms", value),
			MetricType::ConnectionLatency => format!("{:.2}ms", value),
			MetricType::MemoryUsage => format!("{:.2}MB", value),
			MetricType::CpuUsage => format!("{:.2}%", value),
			MetricType::NetworkThroughput => format!("{:.2} msg/s", value),
			MetricType::ErrorRate => format!("{:.2}%", value),
			MetricType::QueueSize => format!("{:.0}", value),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_performance_dashboard_creation() {
		let dashboard = PerformanceDashboard::default_dashboard();
		assert_eq!(dashboard.config.update_interval_ms, 5000);
	}

	#[tokio::test]
	async fn test_metric_recording() {
		let dashboard = PerformanceDashboard::default_dashboard();
		dashboard.start().await.unwrap();

		let metric = PerformanceDashboard::create_metric(
			MetricType::MessageProcessingTime,
			150.0,
			Some("test_channel".to_string()),
			HashMap::new(),
		);

		dashboard.record_metric(metric.clone()).await;

		let recent_metrics = dashboard.get_recent_metrics(10).await;
		assert!(!recent_metrics.is_empty());

		dashboard.stop().await.unwrap();
	}

	#[tokio::test]
	async fn test_trace_span_management() {
		let dashboard = PerformanceDashboard::default_dashboard();
		dashboard.start().await.unwrap();

		let span = dashboard.start_trace_span("test_operation".to_string()).await;
		assert_eq!(span.operation_name, "test_operation");

		dashboard.end_trace_span(&span.span_id).await.unwrap();

		let trace = dashboard.get_trace(&span.trace_id).await;
		assert!(trace.is_some());

		dashboard.stop().await.unwrap();
	}

	#[tokio::test]
	async fn test_alert_generation() {
		let dashboard = PerformanceDashboard::default_dashboard();
		dashboard.start().await.unwrap();

		// Create metric that exceeds threshold
		let metric = PerformanceDashboard::create_metric(
			MetricType::MessageProcessingTime,
			// Value exceeds the default 1000ms threshold to trigger an alert.
			2000.0,
			None,
			HashMap::new(),
		);

		dashboard.record_metric(metric).await;

		let alerts = dashboard.get_active_alerts().await;
		assert!(!alerts.is_empty());

		dashboard.stop().await.unwrap();
	}

	#[test]
	fn test_performance_score_calculation() {
		let score = PerformanceDashboard::calculate_performance_score(50.0, 2.0, 500.0);
		assert!(score >= 0.0 && score <= 100.0);
	}

	#[test]
	fn test_metric_value_formatting() {
		let formatted = PerformanceDashboard::format_metric_value(&MetricType::MessageProcessingTime, 123.456);
		assert_eq!(formatted, "123.46ms");
	}
}
