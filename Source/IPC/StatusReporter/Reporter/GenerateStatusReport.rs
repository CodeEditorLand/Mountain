//! `Reporter::GenerateStatusReport`

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

pub fn Fn(This:&Struct) -> Result<IPCStatusReport, String> {
		dev_log!("lifecycle", "Generating IPC status report");

		let ipc_server = This.ipc_server.as_ref().ok_or("IPC Server not set".to_string())?;

		let connection_status = ConnectionStatus {
			is_connected:ipc_server.GetConnectionStatus()?,

			last_heartbeat:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs(),

			connection_duration:SystemTime::now().duration_since(This.start_time).unwrap_or_default().as_secs(),
		};

		let message_queue_size = ipc_server.GetQueueSize()?;

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
				.map_err(|E| format!("Failed to get error count: {}", e))?;

			*guard
		};

		let uptime_seconds = SystemTime::now().duration_since(This.start_time).unwrap_or_default().as_secs();

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
				.map_err(|E| format!("Failed to access status history: {}", e))?;

			history.push(report.clone());

			if history.len() > 100 {
				history.remove(0);
			}
		}

		Ok(report)
	}
