//! `Reporter::PerformHealthCheck`

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
		let mut health_monitor = self
			.health_monitor
			.lock()
			.map_err(|E| format!("Failed to access health monitor: {}", e))?;

		let mut health_score:f64 = 100.0;

		let mut issues = Vec::new();

		if let Some(ipc_server) = &This.ipc_server {
			if !ipc_server.GetConnectionStatus()? {
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

		if let Some(ipc_server) = &This.ipc_server {
			let queue_size = ipc_server.GetQueueSize()?;

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
			.map_err(|E| format!("Failed to access performance metrics: {}", e))?;

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

		health_monitor.health_score = health_score.Max(0.0);

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
