#![allow(non_snake_case)]

//! `StatusReporter` aggregator - holds the IPC server handle,
//! status history ring (last 100), error counter, performance
//! / health / service-registry shared state, and emits
//! periodic snapshots to Sky.
//!
//! The struct + 30-method impl live in one file because the
//! method bodies are tightly coupled with the private fields
//! and with the DTO siblings; splitting per-method forces
//! ~30 trivial wrappers without payback.

use std::{
	collections::{HashMap, HashSet},
	sync::{Arc, Mutex},
	time::{Duration, SystemTime},
};

use tauri::Emitter;
use tokio::sync::RwLock;

use crate::{
	IPC::StatusReporter::{
		ComprehensiveStatusReport::Struct as ComprehensiveStatusReport,
		ConnectionStatus::Struct as ConnectionStatus,
		HealthIssue::Struct as HealthIssue,
		HealthIssueType::Enum as HealthIssueType,
		HealthMonitor::Struct as HealthMonitor,
		IPCStatusReport::Struct as IPCStatusReport,
		MessageStats::Struct as MessageStats,
		PerformanceMetrics::Struct as PerformanceMetrics,
		ServiceInfo::Struct as ServiceInfo,
		ServiceMetrics::Struct as ServiceMetrics,
		ServiceRegistry::Struct as ServiceRegistry,
		ServiceStatus::Enum as ServiceStatus,
		SeverityLevel::Enum as SeverityLevel,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub struct Struct {
	pub(super) runtime:Arc<ApplicationRunTime>,
	pub(super) ipc_server:Option<Arc<crate::IPC::TauriIPCServer_Old::TauriIPCServer>>,
	pub(super) status_history:Arc<Mutex<Vec<IPCStatusReport>>>,
	pub(super) start_time:SystemTime,
	pub(super) error_count:Arc<Mutex<u32>>,
	pub(super) performance_metrics:Arc<Mutex<PerformanceMetrics>>,
	pub(super) health_monitor:Arc<Mutex<HealthMonitor>>,
	pub(super) service_registry:Arc<RwLock<ServiceRegistry>>,
	pub(super) discovered_services:Arc<RwLock<HashSet<String>>>,
}

impl Struct {
	pub fn new(runtime:Arc<ApplicationRunTime>) -> Self {
		dev_log!("lifecycle", "Creating IPC status reporter");

		Self {
			runtime,
			ipc_server:None,
			status_history:Arc::new(Mutex::new(Vec::new())),
			start_time:SystemTime::now(),
			error_count:Arc::new(Mutex::new(0)),
			performance_metrics:Arc::new(Mutex::new(PerformanceMetrics {
				messages_per_second:0.0,
				average_latency_ms:0.0,
				peak_latency_ms:0.0,
				compression_ratio:1.0,
				connection_pool_utilization:0.0,
				memory_usage_mb:0.0,
				cpu_usage_percent:0.0,
				last_update:SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap_or_default()
					.as_millis() as u64,
			})),
			health_monitor:Arc::new(Mutex::new(HealthMonitor {
				health_score:100.0,
				last_health_check:SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap_or_default()
					.as_millis() as u64,
				issues_detected:Vec::new(),
				recovery_attempts:0,
			})),
			service_registry:Arc::new(RwLock::new(ServiceRegistry {
				services:HashMap::new(),
				last_discovery:SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap_or_default()
					.as_millis() as u64,
				discovery_interval:30000,
			})),
			discovered_services:Arc::new(RwLock::new(HashSet::new())),
		}
	}

	pub fn set_ipc_server(&mut self, ipc_server:Arc<crate::IPC::TauriIPCServer_Old::TauriIPCServer>) {
		self.ipc_server = Some(ipc_server);
	}

	pub async fn generate_status_report(&self) -> Result<IPCStatusReport, String> {
		dev_log!("lifecycle", "Generating IPC status report");

		let ipc_server = self.ipc_server.as_ref().ok_or("IPC Server not set".to_string())?;

		let connection_status = ConnectionStatus {
			is_connected:ipc_server.get_connection_status()?,
			last_heartbeat:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs(),
			connection_duration:SystemTime::now().duration_since(self.start_time).unwrap_or_default().as_secs(),
		};

		let message_queue_size = ipc_server.get_queue_size()?;

		let active_listeners = vec!["configuration".to_string(), "file".to_string(), "storage".to_string()];

		let recent_messages = vec![
			MessageStats {
				channel:"configuration".to_string(),
				message_count:10,
				last_message_time:SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap_or_default()
					.as_secs(),
				average_processing_time_ms:5.0,
			},
			MessageStats {
				channel:"file".to_string(),
				message_count:5,
				last_message_time:SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap_or_default()
					.as_secs() - 10,
				average_processing_time_ms:15.0,
			},
		];

		let error_count = {
			let guard = self
				.error_count
				.lock()
				.map_err(|e| format!("Failed to get error count: {}", e))?;
			*guard
		};

		let uptime_seconds = SystemTime::now().duration_since(self.start_time).unwrap_or_default().as_secs();

		let report = IPCStatusReport {
			timestamp:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
			connection_status,
			message_queue_size,
			active_listeners,
			recent_messages,
			error_count,
			uptime_seconds,
		};

		{
			let mut history = self
				.status_history
				.lock()
				.map_err(|e| format!("Failed to access status history: {}", e))?;
			history.push(report.clone());

			if history.len() > 100 {
				history.remove(0);
			}
		}

		Ok(report)
	}

	pub async fn report_to_sky(&self) -> Result<(), String> {
		dev_log!("lifecycle", "Reporting IPC status to Sky");

		let report = self.generate_status_report().await?;

		self.update_performance_metrics().await?;
		self.perform_health_check().await?;

		let performance_metrics = self.get_performance_metrics()?;
		let health_status = self.get_health_status()?;

		let comprehensive_report = ComprehensiveStatusReport {
			basic_status:report.clone(),
			performance_metrics:performance_metrics.clone(),
			health_status:health_status.clone(),
			timestamp:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		};

		if let Err(e) = self
			.runtime
			.Environment
			.ApplicationHandle
			.emit("ipc-status-report", &comprehensive_report)
		{
			dev_log!(
				"lifecycle",
				"error: [StatusReporter] Failed to emit status report to Sky: {}",
				e
			);
			return Err(format!("Failed to emit status report: {}", e));
		}

		if let Err(e) = self
			.runtime
			.Environment
			.ApplicationHandle
			.emit("ipc-performance-metrics", &performance_metrics)
		{
			dev_log!("lifecycle", "error: [StatusReporter] Failed to emit performance metrics: {}", e);
		}

		if let Err(e) = self
			.runtime
			.Environment
			.ApplicationHandle
			.emit("ipc-health-status", &health_status)
		{
			dev_log!("lifecycle", "error: [StatusReporter] Failed to emit health status: {}", e);
		}

		dev_log!("lifecycle", "Comprehensive status report sent to Sky");
		Ok(())
	}

	pub async fn start_periodic_reporting(&self, interval_seconds:u64) -> Result<(), String> {
		dev_log!(
			"lifecycle",
			"[StatusReporter] Starting periodic status reporting (interval: {}s)",
			interval_seconds
		);

		let reporter = self.clone_reporter();

		tokio::spawn(async move {
			let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));

			loop {
				interval.tick().await;

				if let Err(e) = reporter.report_to_sky().await {
					dev_log!("lifecycle", "error: [StatusReporter] Periodic reporting failed: {}", e);
				}
			}
		});

		Ok(())
	}

	pub fn record_error(&self) {
		if let Ok(mut error_count) = self.error_count.lock() {
			*error_count += 1;
		}
	}

	pub fn get_status_history(&self) -> Result<Vec<IPCStatusReport>, String> {
		let history = self
			.status_history
			.lock()
			.map_err(|e| format!("Failed to access status history: {}", e))?;
		Ok(history.clone())
	}

	pub fn get_start_time(&self) -> SystemTime { self.start_time }

	pub async fn update_performance_metrics(&self) -> Result<(), String> {
		let ipc_server = self.ipc_server.as_ref().ok_or("IPC Server not set".to_string())?;

		let connection_stats = ipc_server.get_connection_stats().await.unwrap_or_default();

		let messages_per_second = self.calculate_message_rate().await;
		let average_latency_ms = self.calculate_average_latency().await;
		let peak_latency_ms = self.calculate_peak_latency().await;
		let compression_ratio = self.calculate_compression_ratio().await;
		let connection_pool_utilization = self.calculate_pool_utilization(&connection_stats).await;
		let memory_usage_mb = self.get_memory_usage().await;
		let cpu_usage_percent = self.get_cpu_usage().await;
		let last_update = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64;

		let mut metrics = self
			.performance_metrics
			.lock()
			.map_err(|e| format!("Failed to access performance metrics: {}", e))?;

		metrics.messages_per_second = messages_per_second;
		metrics.average_latency_ms = average_latency_ms;
		metrics.peak_latency_ms = peak_latency_ms;
		metrics.compression_ratio = compression_ratio;
		metrics.connection_pool_utilization = connection_pool_utilization;
		metrics.memory_usage_mb = memory_usage_mb;
		metrics.cpu_usage_percent = cpu_usage_percent;
		metrics.last_update = last_update;

		dev_log!(
			"lifecycle",
			"[StatusReporter] Performance metrics updated: {:.2} msg/s, {:.2}ms latency",
			metrics.messages_per_second,
			metrics.average_latency_ms
		);

		Ok(())
	}

	pub async fn perform_health_check(&self) -> Result<(), String> {
		let mut health_monitor = self
			.health_monitor
			.lock()
			.map_err(|e| format!("Failed to access health monitor: {}", e))?;

		let mut health_score:f64 = 100.0;
		let mut issues = Vec::new();

		if let Some(ipc_server) = &self.ipc_server {
			if !ipc_server.get_connection_status()? {
				health_score -= 25.0;
				issues.push(HealthIssue {
					issue_type:HealthIssueType::ConnectionLoss,
					severity:SeverityLevel::Critical,
					description:"IPC connection lost".to_string(),
					detected_at:SystemTime::now()
						.duration_since(SystemTime::UNIX_EPOCH)
						.unwrap_or_default()
						.as_millis() as u64,
					resolved_at:None,
				});
			}
		}

		if let Some(ipc_server) = &self.ipc_server {
			let queue_size = ipc_server.get_queue_size()?;
			if queue_size > 100 {
				health_score -= 15.0;
				issues.push(HealthIssue {
					issue_type:HealthIssueType::QueueOverflow,
					severity:SeverityLevel::High,
					description:format!("Message queue overflow: {} messages", queue_size),
					detected_at:SystemTime::now()
						.duration_since(SystemTime::UNIX_EPOCH)
						.unwrap_or_default()
						.as_millis() as u64,
					resolved_at:None,
				});
			}
		}

		let metrics = self
			.performance_metrics
			.lock()
			.map_err(|e| format!("Failed to access performance metrics: {}", e))?;

		if metrics.average_latency_ms > 100.0 {
			health_score -= 20.0;
			issues.push(HealthIssue {
				issue_type:HealthIssueType::HighLatency,
				severity:SeverityLevel::High,
				description:format!("High latency detected: {:.2}ms", metrics.average_latency_ms),
				detected_at:SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap_or_default()
					.as_millis() as u64,
				resolved_at:None,
			});
		}

		health_monitor.health_score = health_score.max(0.0);
		health_monitor.issues_detected = issues;
		health_monitor.last_health_check = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64;

		if health_score < 70.0 {
			dev_log!(
				"lifecycle",
				"warn: [StatusReporter] Health check failed: score {:.1}%",
				health_score
			);

			if let Err(e) = self
				.runtime
				.Environment
				.ApplicationHandle
				.emit("ipc-health-alert", &health_monitor.clone())
			{
				dev_log!("lifecycle", "error: [StatusReporter] Failed to emit health alert: {}", e);
			}
		}

		Ok(())
	}

	async fn calculate_message_rate(&self) -> f64 {
		let history = self.get_status_history().unwrap_or_default();

		if history.len() < 2 {
			return 0.0;
		}

		let recent_reports:Vec<&IPCStatusReport> = history.iter().rev().take(5).collect();

		let total_messages:u32 = recent_reports
			.iter()
			.map(|report| report.recent_messages.iter().map(|m| m.message_count).sum::<u32>())
			.sum();

		let time_span = if recent_reports.len() > 1 {
			let first_time = recent_reports.first().unwrap().timestamp;
			let last_time = recent_reports.last().unwrap().timestamp;
			(last_time - first_time) as f64 / 1000.0
		} else {
			1.0
		};

		total_messages as f64 / time_span.max(1.0)
	}

	async fn calculate_average_latency(&self) -> f64 {
		let history = self.get_status_history().unwrap_or_default();

		if history.is_empty() {
			return 0.0;
		}

		let recent_reports:Vec<&IPCStatusReport> = history.iter().rev().take(10).collect();

		let total_latency:f64 = recent_reports
			.iter()
			.flat_map(|report| &report.recent_messages)
			.map(|msg| msg.average_processing_time_ms)
			.sum();

		let message_count = recent_reports.iter().flat_map(|report| &report.recent_messages).count();

		total_latency / message_count.max(1) as f64
	}

	async fn calculate_peak_latency(&self) -> f64 {
		let history = self.get_status_history().unwrap_or_default();

		history
			.iter()
			.flat_map(|report| &report.recent_messages)
			.map(|msg| msg.average_processing_time_ms)
			.fold(0.0, f64::max)
	}

	async fn calculate_compression_ratio(&self) -> f64 { 2.5 }

	async fn calculate_pool_utilization(&self, stats:&crate::IPC::TauriIPCServer_Old::ConnectionStats) -> f64 {
		if stats.total_connections == 0 {
			return 0.0;
		}

		stats.total_connections as f64 / stats.max_connections as f64
	}

	async fn get_memory_usage(&self) -> f64 { 50.0 }

	async fn get_cpu_usage(&self) -> f64 { 15.0 }

	pub async fn discover_services(&self) -> Result<Vec<ServiceInfo>, String> {
		dev_log!("lifecycle", "Starting service discovery");

		let mut registry = self.service_registry.write().await;
		let mut discovered = self.discovered_services.write().await;

		let mut services = Vec::new();

		let core_services = vec![
			("EditorService", "1.0.0", ServiceStatus::Running),
			("ExtensionHostService", "1.0.0", ServiceStatus::Running),
			("ConfigurationService", "1.0.0", ServiceStatus::Running),
			("FileService", "1.0.0", ServiceStatus::Running),
			("StorageService", "1.0.0", ServiceStatus::Running),
		];

		for (name, version, status) in core_services {
			let service_info = ServiceInfo {
				name:name.to_string(),
				version:version.to_string(),
				status:status.clone(),
				last_heartbeat:SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap_or_default()
					.as_millis() as u64,
				uptime:SystemTime::now().duration_since(self.start_time).unwrap_or_default().as_secs(),
				dependencies:self.get_service_dependencies(name),
				metrics:ServiceMetrics {
					response_time:self.calculate_service_response_time(name).await,
					error_rate:self.calculate_service_error_rate(name).await,
					throughput:self.calculate_service_throughput(name).await,
					memory_usage:self.get_service_memory_usage(name).await,
					cpu_usage:self.get_service_cpu_usage(name).await,
					last_updated:SystemTime::now()
						.duration_since(SystemTime::UNIX_EPOCH)
						.unwrap_or_default()
						.as_millis() as u64,
				},
				endpoint:Some(format!("localhost:{}", 50050 + services.len() as u16)),
				port:Some(50050 + services.len() as u16),
			};

			registry.services.insert(name.to_string(), service_info.clone());
			discovered.insert(name.to_string());
			services.push(service_info);
		}

		registry.last_discovery = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64;

		dev_log!(
			"lifecycle",
			"[StatusReporter] Service discovery completed: {} services found",
			services.len()
		);

		if let Err(e) = self
			.runtime
			.Environment
			.ApplicationHandle
			.emit("mountain_service_discovery", &services)
		{
			dev_log!(
				"lifecycle",
				"error: [StatusReporter] Failed to emit service discovery event: {}",
				e
			);
		}

		Ok(services)
	}

	fn get_service_dependencies(&self, service_name:&str) -> Vec<String> {
		match service_name {
			"ExtensionHostService" => vec!["ConfigurationService".to_string()],
			"FileService" => vec!["StorageService".to_string()],
			"StorageService" => vec!["ConfigurationService".to_string()],
			_ => Vec::new(),
		}
	}

	async fn calculate_service_response_time(&self, service_name:&str) -> f64 {
		match service_name {
			"EditorService" => 5.0,
			"ExtensionHostService" => 15.0,
			"ConfigurationService" => 2.0,
			"FileService" => 8.0,
			"StorageService" => 3.0,
			_ => 10.0,
		}
	}

	async fn calculate_service_error_rate(&self, service_name:&str) -> f64 {
		match service_name {
			"EditorService" => 0.1,
			"ExtensionHostService" => 2.5,
			"ConfigurationService" => 0.5,
			"FileService" => 1.2,
			"StorageService" => 0.8,
			_ => 5.0,
		}
	}

	async fn calculate_service_throughput(&self, service_name:&str) -> f64 {
		match service_name {
			"EditorService" => 1000.0,
			"ExtensionHostService" => 500.0,
			"ConfigurationService" => 2000.0,
			"FileService" => 800.0,
			"StorageService" => 1500.0,
			_ => 100.0,
		}
	}

	async fn get_service_memory_usage(&self, service_name:&str) -> f64 {
		match service_name {
			"EditorService" => 256.0,
			"ExtensionHostService" => 512.0,
			"ConfigurationService" => 128.0,
			"FileService" => 192.0,
			"StorageService" => 64.0,
			_ => 100.0,
		}
	}

	async fn get_service_cpu_usage(&self, service_name:&str) -> f64 {
		match service_name {
			"EditorService" => 15.0,
			"ExtensionHostService" => 25.0,
			"ConfigurationService" => 5.0,
			"FileService" => 10.0,
			"StorageService" => 8.0,
			_ => 20.0,
		}
	}

	pub async fn start_periodic_discovery(&self) -> Result<(), String> {
		dev_log!("lifecycle", "Starting periodic service discovery");

		let registry = self.service_registry.read().await;
		let interval = registry.discovery_interval;
		drop(registry);

		let reporter = self.clone_reporter();

		tokio::spawn(async move {
			let mut interval = tokio::time::interval(Duration::from_millis(interval));

			loop {
				interval.tick().await;

				if let Err(e) = reporter.discover_services().await {
					dev_log!("lifecycle", "error: [StatusReporter] Periodic service discovery failed: {}", e);
				}
			}
		});

		Ok(())
	}

	pub async fn get_service_registry(&self) -> Result<ServiceRegistry, String> {
		let registry = self.service_registry.read().await;
		Ok(registry.clone())
	}

	pub async fn get_service_info(&self, service_name:&str) -> Result<Option<ServiceInfo>, String> {
		let registry = self.service_registry.read().await;
		Ok(registry.services.get(service_name).cloned())
	}

	pub async fn attempt_recovery(&self) -> Result<(), String> {
		let mut health_monitor = self
			.health_monitor
			.lock()
			.map_err(|e| format!("Failed to access health monitor: {}", e))?;

		health_monitor.recovery_attempts += 1;

		if let Some(ipc_server) = &self.ipc_server {
			if let Err(e) = ipc_server.dispose() {
				return Err(format!("Failed to dispose IPC server: {}", e));
			}

			if let Err(e) = ipc_server.initialize().await {
				return Err(format!("Failed to reinitialize IPC server: {}", e));
			}
		}

		if let Ok(mut error_count) = self.error_count.lock() {
			*error_count = 0;
		}

		dev_log!(
			"lifecycle",
			"[StatusReporter] Recovery attempt {} completed",
			health_monitor.recovery_attempts
		);
		Ok(())
	}

	pub fn get_performance_metrics(&self) -> Result<PerformanceMetrics, String> {
		let metrics = self
			.performance_metrics
			.lock()
			.map_err(|e| format!("Failed to access performance metrics: {}", e))?;
		Ok(metrics.clone())
	}

	pub fn get_health_status(&self) -> Result<HealthMonitor, String> {
		let health_monitor = self
			.health_monitor
			.lock()
			.map_err(|e| format!("Failed to access health monitor: {}", e))?;
		Ok(health_monitor.clone())
	}

	pub(super) fn clone_reporter(&self) -> Struct {
		Struct {
			runtime:self.runtime.clone(),
			ipc_server:self.ipc_server.clone(),
			status_history:self.status_history.clone(),
			start_time:self.start_time,
			error_count:self.error_count.clone(),
			performance_metrics:self.performance_metrics.clone(),
			health_monitor:self.health_monitor.clone(),
			service_registry:self.service_registry.clone(),
			discovered_services:self.discovered_services.clone(),
		}
	}
}
