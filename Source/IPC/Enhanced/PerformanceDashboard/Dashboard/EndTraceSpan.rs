//! `Dashboard::EndTraceSpan`

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

pub fn Fn(This:&Struct, span_id:&str) -> Result<(), String> {
	let mut traces = This.traces.write().await;

	if let Some(span) = traces.get_mut(span_id) {
		let end_time = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64;

		span.end_time = Some(end_time);

		span.duration_ms = Some(end_time.saturating_sub(span.start_time));

		dev_log!(
			"ipc",
			"[PerformanceDashboard] Ended trace span: {} (duration: {}ms)",
			span.operation_name,
			span.duration_ms.unwrap_or(0)
		);

		Ok(())
	} else {
		Err(format!("Trace span not found: {}", span_id))
	}
}
