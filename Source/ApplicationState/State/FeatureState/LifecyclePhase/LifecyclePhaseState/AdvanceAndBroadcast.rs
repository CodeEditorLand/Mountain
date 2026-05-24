//! `LifecyclePhaseState::AdvanceAndBroadcast`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use tokio::sync::Notify;
use CommonLibrary::IPC::SkyEvent::SkyEvent;
use crate::{IPC::SkyEmit::Fn, dev_log};

pub fn Fn<R:tauri::Runtime>(&self, NewPhase:Phase, ApplicationHandle:&tauri::AppHandle<R>) {
		// Local `use tauri::Emitter` removed - now routed through
		// `LogSkyEmit` which carries the trait import internally.
		let Previous = This.GetPhase();

		if NewPhase <= Previous {
			return;
		}

		This.SetPhase(NewPhase);

		let Label = match NewPhase {
			1 => "Starting",

			2 => "Ready",

			3 => "Restored",

			4 => "Eventually",

			_ => "Unknown",
		};

		if let Err(Error) = LogSkyEmit(
			ApplicationHandle,
			SkyEvent::LifecyclePhaseChanged.AsStr(),
			serde_json::json!({
				"phase": NewPhase,
				"previous": Previous,
				"label": Label,
			}),
		) {
			dev_log!(
				"lifecycle",
				"warn: [LifecyclePhaseState] sky://lifecycle/phaseChanged emit failed: {}",
				Error
			);
		}
	}
