//! `Features::StartMonitoring`

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

pub fn Fn(This:&Struct) -> Result<(), String> {
	dev_log!("lifecycle", "Starting advanced monitoring");

	let features1 = This.clone_features();

	let features2 = This.clone_features();

	let features3 = This.clone_features();

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
