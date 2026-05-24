//! `Reporter::New`

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

pub fn Fn(runtime:Arc<ApplicationRunTime>) -> Struct {
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
