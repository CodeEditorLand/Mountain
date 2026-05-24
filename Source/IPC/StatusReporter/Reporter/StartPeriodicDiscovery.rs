//! `Reporter::StartPeriodicDiscovery`

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
		dev_log!("lifecycle", "Starting periodic service discovery");

		let registry = This.service_registry.read().await;

		let interval = registry.discovery_interval;

		drop(registry);

		let reporter = This.clone_reporter();

		tokio::spawn(async move {
			let mut interval = tokio::time::interval(Duration::from_millis(interval));

			loop {
				interval.tick().await;

				if let Err(e) = reporter.DiscoverServices().await {
					dev_log!("lifecycle", "error: [StatusReporter] Periodic service discovery failed: {}", e);
				}
			}
		});

		Ok(())
	}
