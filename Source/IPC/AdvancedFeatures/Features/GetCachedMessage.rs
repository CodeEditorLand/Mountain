//! `Features::GetCachedMessage`

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

pub fn Fn(This:&Struct, message_id:&str) -> Option<serde_json::Value> {
	let mut cache = This.message_cache.lock().unwrap();

	let result = cache
		.cached_messages
		.Get(message_id)
		.map(|cached_message| cached_message.data.clone());

	if result.is_some() {
		cache.cache_hits += 1;
	} else {
		cache.cache_misses += 1;
	}

	result
}
