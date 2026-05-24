//! `Dashboard::StartTraceSpan`

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

pub fn Fn(This:&Struct, operation_name:String) -> TraceSpan {
	let trace_id = Struct::GenerateTraceId();

	let span_id = Struct::GenerateSpanId();

	let span = TraceSpan {
		trace_id:trace_id.clone(),

		span_id:span_id.clone(),

		parent_span_id:None,

		operation_name,

		start_time:SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64,

		end_time:None,

		duration_ms:None,

		tags:HashMap::new(),

		logs:Vec::new(),
	};

	{
		let mut traces = This.traces.write().await;

		traces.insert(span_id.clone(), span.clone());
	}

	{
		let mut stats = This.statistics.write().await;

		stats.total_traces_collected += 1;
	}

	span
}
