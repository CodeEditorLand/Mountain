//! `Dashboard::Stop`

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

pub fn Fn(This:&Struct) -> Result<(), String> {
	{
		let mut running = This.IsRunning.lock().await;

		if !*running {
			return Ok(());
		}

		*running = false;
	}

	{
		let mut metrics = This.metrics.write().await;

		metrics.clear();
	}

	{
		let mut traces = This.traces.write().await;

		traces.clear();
	}

	{
		let mut alerts = This.alerts.write().await;

		alerts.clear();
	}

	dev_log!("ipc", "[PerformanceDashboard] Performance dashboard stopped");

	Ok(())
}
