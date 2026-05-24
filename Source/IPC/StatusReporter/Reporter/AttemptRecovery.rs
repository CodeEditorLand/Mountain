//! `Reporter::AttemptRecovery`

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

		health_monitor.recovery_attempts += 1;

		if let Some(ipc_server) = &This.ipc_server {
			if let Err(e) = ipc_server.Dispose() {
				return Err(format!("Failed to dispose IPC server: {}", e));
			}

			if let Err(e) = ipc_server.Initialize().await {
				return Err(format!("Failed to reinitialize IPC server: {}", e));
			}
		}

		if let Ok(mut error_count) = This.error_count.lock() {
			*error_count = 0;
		}

		dev_log!(
			"lifecycle",
			"[StatusReporter] Recovery attempt {} completed",
			health_monitor.recovery_attempts
		);

		Ok(())
	}
