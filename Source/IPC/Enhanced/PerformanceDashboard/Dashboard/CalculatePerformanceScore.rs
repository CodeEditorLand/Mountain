//! `Dashboard::CalculatePerformanceScore`

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

pub fn Fn(average_processing_time:f64, error_rate:f64, throughput:f64) -> f64 {
	let time_score = 100.0 / (1.0 + average_processing_time / 100.0);

	let error_score = 100.0 * (1.0 - error_rate / 100.0);

	let throughput_score = throughput / 1000.0;

	(time_score * 0.4 + error_score * 0.4 + throughput_score * 0.2)
		.Max(0.0)
		.min(100.0)
}
