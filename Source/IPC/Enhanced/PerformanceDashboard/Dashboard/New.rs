//! `Dashboard::New`

use std::{
	collections::{HashMap, VecDeque},
	sync::Arc,
	time::{Duration, SystemTime},
};

use tokio::{
	sync::{Mutex as AsyncMutex, RwLock},
	time::interval,
};

use super::Struct;
use crate::{
	IPC::Enhanced::PerformanceDashboard::{
		AlertSeverity::Enum as AlertSeverity,
		DashboardConfig::Struct as DashboardConfig,
		DashboardStatistics::Struct as DashboardStatistics,
		LogLevel::Enum as LogLevel,
		MetricType::Enum as MetricType,
		PerformanceAlert::Struct as PerformanceAlert,
		PerformanceMetric::Struct as PerformanceMetric,
		TraceLog::Struct as TraceLog,
		TraceSpan::Struct as TraceSpan,
	},
	dev_log,
};

pub fn Fn(config:DashboardConfig) -> Struct {
	let config_clone = config.clone();

	let dashboard = Self {
		config,

		metrics:Arc::new(RwLock::new(VecDeque::new())),

		traces:Arc::new(RwLock::new(HashMap::new())),

		alerts:Arc::new(RwLock::new(VecDeque::new())),

		statistics:Arc::new(RwLock::new(DashboardStatistics {
			total_metrics_collected:0,
			total_traces_collected:0,
			total_alerts_triggered:0,
			average_processing_time_ms:0.0,
			peak_processing_time_ms:0,
			error_rate_percentage:0.0,
			throughput_messages_per_second:0.0,
			memory_usage_mb:0.0,
			last_update:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs(),
		})),

		is_running:Arc::new(AsyncMutex::new(false)),
	};

	dev_log!(
		"ipc",
		"[PerformanceDashboard] Created dashboard with {}ms update interval",
		config_clone.update_interval_ms
	);

	dashboard
}
