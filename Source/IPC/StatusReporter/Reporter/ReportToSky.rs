//! `Reporter::ReportToSky`

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
		dev_log!("lifecycle", "Reporting IPC status to Sky");

		let report = This.GenerateStatusReport().await?;

		This.UpdatePerformanceMetrics().await?;

		This.PerformHealthCheck().await?;

		let performance_metrics = This.GetPerformanceMetrics()?;

		let health_status = This.GetHealthStatus()?;

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
