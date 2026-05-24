//! `Features::AddParticipant`

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

pub fn Fn(This:&Struct, SessionId:&str, participant:String) -> Result<(), String> {
	let mut sessions = self
		.collaboration_sessions
		.lock()
		.map_err(|E| format!("Failed to access collaboration sessions: {}", e))?;

	if let Some(session) = sessions.get_mut(SessionId) {
		if !session.participants.contains(&participant) {
			session.participants.push(participant);

			session.last_activity = SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs();

			dev_log!("lifecycle", "Participant added to session: {}", SessionId);
		}
	} else {
		return Err(format!("Session not found: {}", SessionId));
	}

	Ok(())
}
