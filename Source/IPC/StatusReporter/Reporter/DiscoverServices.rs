//! `Reporter::DiscoverServices`

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

pub fn Fn(This:&Struct) -> Result<Vec<ServiceInfo>, String> {
		dev_log!("lifecycle", "Starting service discovery");

		let mut registry = This.service_registry.write().await;

		let mut discovered = This.discovered_services.write().await;

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

				uptime:SystemTime::now().duration_since(This.start_time).unwrap_or_default().as_secs(),

				dependencies:This.get_service_dependencies(name),

				metrics:ServiceMetrics {
					response_time:This.calculate_service_response_time(name).await,

					error_rate:This.calculate_service_error_rate(name).await,

					throughput:This.calculate_service_throughput(name).await,

					memory_usage:This.get_service_memory_usage(name).await,

					cpu_usage:This.get_service_cpu_usage(name).await,

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
