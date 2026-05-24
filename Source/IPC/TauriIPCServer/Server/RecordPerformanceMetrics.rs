//! `Server::RecordPerformanceMetrics`

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

use tauri::{AppHandle, Emitter, Manager};

use super::{
	super::{
		Connection::{ConnectionManager, ConnectionStats},
		Encryption::{SecureMessageChannel, Struct},
		Message::{ConnectionStatus, ListenerCallback, TauriIPCMessage},
		Security::PermissionManager::{
			Manager::Struct as PermissionManager,
			SecurityContext::Struct as SecurityContext,
			SecurityEvent::Struct as SecurityEvent,
			SecurityEventType::Enum as SecurityEventType,
		},
	},
	Struct,
};
use crate::dev_log;

pub fn Fn(&self, channel:String, duration:std::time::Duration, success:bool) {
	dev_log!(
		"ipc",
		"[TauriIPCServer] Performance recorded - Channel: {}, Duration: {:?}, Success: {}",
		channel,
		duration,
		success
	);

	// This would integrate with PerformanceDashboard in the future
}
