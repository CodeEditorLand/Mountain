//! `AdvancedFeatures` aggregator - holds the runtime handle,
//! cumulative `PerformanceStats::Struct`, the realtime
//! collaboration-session map, and the
//! `MessageCache::Struct`. Spawns three monitor tasks
//! (`monitor_performance`, `cleanup_cache`,
//! `monitor_collaboration_sessions`) on `start_monitoring`.
//! The 12-method impl is kept in one file - tightly-coupled
//! cluster.

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
	time::{Duration, SystemTime},
};

use tauri::Emitter;
use tokio::time::interval;

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

#[derive(Clone)]
pub struct Struct {
	pub(super) runtime:Arc<ApplicationRunTime>,

	pub(super) performance_stats:Arc<Mutex<PerformanceStats>>,

	pub(super) collaboration_sessions:Arc<Mutex<HashMap<String, CollaborationSession>>>,

	pub(super) message_cache:Arc<Mutex<MessageCache>>,
}

impl Struct {
	pub fn new(runtime:Arc<ApplicationRunTime>) -> Self {
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

	pub async fn start_monitoring(&self) -> Result<(), String> {
		dev_log!("lifecycle", "Starting advanced monitoring");

		let features1 = self.clone_features();

		let features2 = self.clone_features();

		let features3 = self.clone_features();

		tokio::spawn(async move {
			features1.monitor_performance().await;
		});

		tokio::spawn(async move {
			features2.cleanup_cache().await;
		});

		tokio::spawn(async move {
			features3.monitor_collaboration_sessions().await;
		});

		Ok(())
	}

	async fn monitor_performance(&self) {
		let mut interval = interval(Duration::from_secs(10));

		loop {
			interval.tick().await;

			let stats = self.calculate_performance_stats().await;

			if let Err(e) = self.runtime.Environment.ApplicationHandle.emit("ipc-performance-stats", &stats) {
				dev_log!("ipc", "error: [AdvancedFeatures] Failed to emit performance stats: {}", e);
			}

			dev_log!("lifecycle", "Performance stats updated");
		}
	}

	async fn calculate_performance_stats(&self) -> PerformanceStats {
		let mut stats = self.performance_stats.lock().unwrap_or_else(|e| e.into_inner());

		stats.connection_uptime = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs()
			- stats.last_update;

		stats.last_update = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();

		stats.clone()
	}

	async fn cleanup_cache(&self) {
		let mut interval = interval(Duration::from_secs(60));

		loop {
			interval.tick().await;

			let current_time = SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs();

			let mut cache = self.message_cache.lock().unwrap_or_else(|e| e.into_inner());

			cache
				.cached_messages
				.retain(|_, cached_message| current_time < cached_message.timestamp + cached_message.ttl);

			cache.cache_size = cache.cached_messages.len();

			dev_log!("lifecycle", "Cache cleaned, {} entries remaining", cache.cache_size);
		}
	}

	async fn monitor_collaboration_sessions(&self) {
		let mut interval = interval(Duration::from_secs(30));

		loop {
			interval.tick().await;

			let current_time = SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs();

			let mut sessions = self.collaboration_sessions.lock().unwrap_or_else(|e| e.into_inner());

			sessions.retain(|_, session| current_time - session.last_activity < 300);

			let active_sessions:Vec<CollaborationSession> = sessions.values().cloned().collect();

			if let Err(e) = self
				.runtime
				.Environment
				.ApplicationHandle
				.emit("collaboration-sessions-update", &active_sessions)
			{
				dev_log!("ipc", "error: [AdvancedFeatures] Failed to emit collaboration sessions: {}", e);
			}

			dev_log!("lifecycle", "Collaboration sessions monitored, {} active", sessions.len());
		}
	}

	pub async fn cache_message(&self, message_id:String, data:serde_json::Value, ttl:u64) -> Result<(), String> {
		let mut cache = self
			.message_cache
			.lock()
			.map_err(|e| format!("Failed to access message cache: {}", e))?;

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

	pub async fn get_cached_message(&self, message_id:&str) -> Option<serde_json::Value> {
		let mut cache = self.message_cache.lock().unwrap_or_else(|e| e.into_inner());

		let result = cache
			.cached_messages
			.get(message_id)
			.map(|cached_message| cached_message.data.clone());

		if result.is_some() {
			cache.cache_hits += 1;
		} else {
			cache.cache_misses += 1;
		}

		result
	}

	pub async fn create_collaboration_session(
		&self,

		session_id:String,

		permissions:CollaborationPermissions,
	) -> Result<(), String> {
		let mut sessions = self
			.collaboration_sessions
			.lock()
			.map_err(|e| format!("Failed to access collaboration sessions: {}", e))?;

		let session = CollaborationSession {
			session_id:session_id.clone(),

			participants:Vec::new(),

			active_documents:Vec::new(),

			last_activity:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs(),

			permissions,
		};

		sessions.insert(session_id, session);

		dev_log!("lifecycle", "Collaboration session created");

		Ok(())
	}

	pub async fn add_participant(&self, session_id:&str, participant:String) -> Result<(), String> {
		let mut sessions = self
			.collaboration_sessions
			.lock()
			.map_err(|e| format!("Failed to access collaboration sessions: {}", e))?;

		if let Some(session) = sessions.get_mut(session_id) {
			if !session.participants.contains(&participant) {
				session.participants.push(participant);

				session.last_activity = SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap_or_default()
					.as_secs();

				dev_log!("lifecycle", "Participant added to session: {}", session_id);
			}
		} else {
			return Err(format!("Session not found: {}", session_id));
		}

		Ok(())
	}

	pub async fn record_message_statistics(&self, sent:bool, processing_time_ms:u64) {
		let mut stats = self.performance_stats.lock().unwrap_or_else(|e| e.into_inner());

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

	pub async fn record_error(&self) {
		let mut stats = self.performance_stats.lock().unwrap_or_else(|e| e.into_inner());

		stats.error_count += 1;
	}

	pub async fn get_performance_stats(&self) -> Result<PerformanceStats, String> {
		Ok(self.calculate_performance_stats().await)
	}

	pub async fn get_cache_stats(&self) -> Result<MessageCache, String> {
		let cache = self.message_cache.lock().unwrap_or_else(|e| e.into_inner());

		Ok(cache.clone())
	}

	pub async fn get_collaboration_sessions(&self) -> Vec<CollaborationSession> {
		let sessions = self.collaboration_sessions.lock().unwrap_or_else(|e| e.into_inner());

		sessions.values().cloned().collect()
	}

	pub(super) fn clone_features(&self) -> Self {
		Self {
			runtime:self.runtime.clone(),

			performance_stats:self.performance_stats.clone(),

			collaboration_sessions:self.collaboration_sessions.clone(),

			message_cache:self.message_cache.clone(),
		}
	}
}
