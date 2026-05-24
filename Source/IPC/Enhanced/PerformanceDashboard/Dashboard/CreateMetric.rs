//! `Dashboard::CreateMetric`

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

pub fn Fn(
	metric_type:MetricType,

	value:f64,

	channel:Option<String>,

	tags:HashMap<String, String>,
) -> PerformanceMetric {
	PerformanceMetric {
		metric_type,

		value,

		timestamp:SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64,

		channel,

		tags,
	}
}
