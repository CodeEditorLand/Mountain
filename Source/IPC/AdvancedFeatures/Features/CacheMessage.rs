//! `Features::CacheMessage`

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

pub fn Fn(This:&Struct, message_id:String, data:serde_json::Value, ttl:u64) -> Result<(), String> {
	let mut cache = self
		.message_cache
		.lock()
		.map_err(|E| format!("Failed to access message cache: {}", e))?;

	let cached_message = CachedMessage {
		data,

		timestamp:SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs(),

		ttl,
	};

	cache.cached_messages.insert(message_id.clone(), cached_message);

	cache.cache_size = cache.cached_messages.len();

	dev_log!("lifecycle", "Message cached: {}, TTL: {}s", message_id, ttl);

	Ok(())
}
