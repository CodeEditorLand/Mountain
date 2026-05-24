pub mod New;
pub mod SetIpcServer;
pub mod GenerateStatusReport;
pub mod ReportToSky;
pub mod StartPeriodicReporting;
pub mod RecordError;
pub mod GetStatusHistory;
pub mod GetStartTime;
pub mod UpdatePerformanceMetrics;
pub mod PerformHealthCheck;
pub mod DiscoverServices;
pub mod StartPeriodicDiscovery;
pub mod GetServiceRegistry;
pub mod GetServiceInfo;
pub mod AttemptRecovery;
pub mod GetPerformanceMetrics;
pub mod GetHealthStatus;

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
