//! `Dashboard::RecordMetric`

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

pub fn Fn(This:&Struct, metric:PerformanceMetric) {
	let mut metrics = This.metrics.write().await;

	metrics.push_back(metric.clone());

	drop(metrics);

	This.update_statistics().await;

	This.check_alerts(&metric).await;

	dev_log!("ipc", "[PerformanceDashboard] Recorded metric: {:?}", metric.metric_type);
}
