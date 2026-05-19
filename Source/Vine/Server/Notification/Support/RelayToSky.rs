#![allow(non_snake_case)]

//! Shared helper for notification atoms that are pure sky-event relays.
//!
//! Many Cocoon → Mountain notification atoms do exactly two things:
//! 1. `ApplicationHandle.emit(sky_event, Parameter)`
//! 2. `dev_log!(tag, "...")`
//!
//! This function collapses that pair so each such atom is a one-liner.
//!
//! - `SkyEvent` - the `sky://…` Tauri event name.
//! - `LogTag`   - the dev-log tag (`"grpc"`, `"output-verbose"`, …).
//! - `LogLine`  - pre-formatted message; skipped when empty.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub fn RelayToSky(Service:&MountainVinegRPCService, SkyEvent:&str, Parameter:&Value, LogTag:&str, LogLine:&str) {
	let _ = Service.ApplicationHandle().emit(SkyEvent, Parameter);

	if !LogLine.is_empty() {
		dev_log!(LogTag, "{}", LogLine);
	}
}
