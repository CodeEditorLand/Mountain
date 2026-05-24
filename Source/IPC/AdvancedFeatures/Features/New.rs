//! `Features::New`

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

pub fn Fn(runtime:Arc<ApplicationRunTime>) -> Struct {
	dev_log!("lifecycle", "Initializing advanced IPC features");

	Self {
		runtime,

		performance_stats:Arc::new(Mutex::new(PerformanceStats {
			total_messages_sent:0,
			total_messages_received:0,
			average_processing_time_ms:0.0,
			peak_message_rate:0,
			error_count:0,
			last_update:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs(),
			connection_uptime:0,
		})),

		collaboration_sessions:Arc::new(Mutex::new(HashMap::new())),

		message_cache:Arc::new(Mutex::new(MessageCache {
			cached_messages:HashMap::new(),
			cache_hits:0,
			cache_misses:0,
			cache_size:0,
		})),
	}
}
