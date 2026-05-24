pub mod New;
pub mod StartMonitoring;
pub mod CacheMessage;
pub mod GetCachedMessage;
pub mod CreateCollaborationSession;
pub mod AddParticipant;
pub mod RecordMessageStatistics;
pub mod RecordError;
pub mod GetPerformanceStats;
pub mod GetCacheStats;
pub mod GetCollaborationSessions;

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
