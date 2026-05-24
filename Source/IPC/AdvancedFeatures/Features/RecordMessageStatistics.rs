//! `Features::RecordMessageStatistics`

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
	time::{Duration, SystemTime},
};

use tauri::Emitter;
use tokio::time::interval;

use super::Struct;
use crate::{
	IPC::AdvancedFeatures::{
		CachedMessage::Struct as CachedMessage,
		CollaborationPermissions::Struct as CollaborationPermissions,
		CollaborationSession::Struct as CollaborationSession,
		MessageCache::Struct as MessageCache,
		PerformanceStats::Struct as PerformanceStats,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub fn Fn(This:&Struct, sent:bool, processing_time_ms:u64) {
	let mut stats = This.performance_stats.lock().unwrap();

	if sent {
		stats.total_messages_sent += 1;
	} else {
		stats.total_messages_received += 1;
	}

	let total_messages = stats.total_messages_sent + stats.total_messages_received;

	stats.average_processing_time_ms = (stats.average_processing_time_ms * (total_messages - 1) as f64
		+ processing_time_ms as f64)
		/ total_messages as f64;
}
