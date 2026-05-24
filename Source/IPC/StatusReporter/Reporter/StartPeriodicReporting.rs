//! `Reporter::StartPeriodicReporting`

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

pub fn Fn(This:&Struct, interval_seconds:u64) -> Result<(), String> {
		dev_log!(
			"lifecycle",
			"[StatusReporter] Starting periodic status reporting (interval: {}s)",
			interval_seconds
		);

		let reporter = This.clone_reporter();

		tokio::spawn(async move {
			let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));

			loop {
				interval.tick().await;

				if let Err(e) = reporter.ReportToSky().await {
					dev_log!("lifecycle", "error: [StatusReporter] Periodic reporting failed: {}", e);
				}
			}
		});

		Ok(())
	}
