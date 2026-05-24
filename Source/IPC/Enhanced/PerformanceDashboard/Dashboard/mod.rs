pub mod New;
pub mod Start;
pub mod Stop;
pub mod RecordMetric;
pub mod StartTraceSpan;
pub mod EndTraceSpan;
pub mod AddTraceLog;
pub mod GetStatistics;
pub mod GetRecentMetrics;
pub mod GetActiveAlerts;
pub mod GetTrace;
pub mod DefaultDashboard;
pub mod HighFrequencyDashboard;
pub mod CreateMetric;
pub mod CreateTraceLog;
pub mod CalculatePerformanceScore;
pub mod FormatMetricValue;

use std::{
	collections::{HashMap, VecDeque},
	sync::Arc,
	time::{Duration, SystemTime},
};

use tokio::{
	sync::{Mutex as AsyncMutex, RwLock},
	time::interval,
};

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

pub struct Struct {
	pub(super) config:DashboardConfig,

	pub(super) metrics:Arc<RwLock<VecDeque<PerformanceMetric>>>,

	pub(super) traces:Arc<RwLock<HashMap<String, TraceSpan>>>,

	pub(super) alerts:Arc<RwLock<VecDeque<PerformanceAlert>>>,

	pub(super) statistics:Arc<RwLock<DashboardStatistics>>,

	pub(super) is_running:Arc<AsyncMutex<bool>>,
}
