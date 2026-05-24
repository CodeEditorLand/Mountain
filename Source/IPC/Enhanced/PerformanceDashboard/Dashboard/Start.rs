//! `Dashboard::Start`

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

		if *running {
			return Ok(());
		}

		*running = true;
	}

	This.start_metrics_collection().await;

	This.start_alert_monitoring().await;

	This.start_data_cleanup().await;

	dev_log!("ipc", "[PerformanceDashboard] Performance dashboard started");

	Ok(())
}
