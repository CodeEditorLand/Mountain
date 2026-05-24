//! `Dashboard::FormatMetricValue`

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

pub fn Fn(metric_type:&MetricType, value:f64) -> String {
	match metric_type {
		MetricType::MessageProcessingTime => format!("{:.2}ms", value),

		MetricType::ConnectionLatency => format!("{:.2}ms", value),

		MetricType::MemoryUsage => format!("{:.2}MB", value),

		MetricType::CpuUsage => format!("{:.2}%", value),

		MetricType::NetworkThroughput => format!("{:.2} msg/s", value),

		MetricType::ErrorRate => format!("{:.2}%", value),

		MetricType::QueueSize => format!("{:.0}", value),
	}
}
