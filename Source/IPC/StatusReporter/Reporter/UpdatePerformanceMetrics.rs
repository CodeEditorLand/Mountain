//! `Reporter::UpdatePerformanceMetrics`

use super::Struct;
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

pub fn Fn(This:&Struct) -> Result<(), String> {
		let ipc_server = This.ipc_server.as_ref().ok_or("IPC Server not set".to_string())?;

		let connection_stats = ipc_server.GetConnectionStats().await.unwrap_or_default();

		let messages_per_second = This.calculate_message_rate().await;

		let average_latency_ms = This.calculate_average_latency().await;

		let peak_latency_ms = This.calculate_peak_latency().await;

		let compression_ratio = This.CalculateCompressionRatio().await;

		let connection_pool_utilization = This.calculate_pool_utilization(&connection_stats).await;

		let memory_usage_mb = This.GetMemoryUsage().await;

		let cpu_usage_percent = This.GetCpuUsage().await;

		let last_update = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64;

		let mut metrics = self
			.performance_metrics
			.lock()
			.map_err(|E| format!("Failed to access performance metrics: {}", e))?;

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
