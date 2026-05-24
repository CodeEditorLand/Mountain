//! `Reporter::SetIpcServer`

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

pub fn Fn(This:&mut Struct, ipc_server:Arc<crate::IPC::TauriIPCServer_Old::TauriIPCServer>) {
		This.ipc_server = Some(ipc_server);
	}
