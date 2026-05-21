#![allow(non_snake_case)]
//! Cocoon → Mountain `terminal.sendText` / `terminal.show` / `terminal.hide` /
//! `terminal.dispose` notifications. Shared atom because the four wire
//! methods all fan through the same `sky://terminal/*` relay and the
//! same provider-side PTY drive, differing only in which provider call
//! fires (sendText vs dispose) and whether the payload carries text.
//!
//! Two concerns per invocation:
//!   1. Notify Sky on `sky://terminal/<suffix>` so the xterm panel can
//!      show/hide / print text / remove the panel.
//!   2. Drive the underlying PTY via the `TerminalProvider` so the OS process
//!      sees the text / receives SIGHUP on dispose.

use std::sync::Arc;

use serde_json::Value;
use tauri::Emitter;
use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn TerminalLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	let EventName = format!("sky://terminal/{}", &MethodName["terminal.".len()..]);

	if let Err(Error) = Service.ApplicationHandle().emit(&EventName, Parameter) {
		dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
	}

	// Terminal handles from Cocoon arrive as `terminal:N`; strip the
	// prefix to recover the numeric identifier the provider expects.
	let HandleNumeric = Parameter
		.get("handle")
		.and_then(|H| H.as_str())
		.and_then(|S| S.trim_start_matches("terminal:").parse::<u64>().ok());

	if let Some(TerminalId) = HandleNumeric {
		let Provider:Arc<dyn TerminalProvider> = Service.RunTime().Environment.Require();

		match MethodName {
			"terminal.sendText" => {
				let Text = Parameter.get("text").and_then(|T| T.as_str()).unwrap_or("").to_string();

				let ProviderForTask = Provider.clone();

				tokio::spawn(async move {
					let _ = ProviderForTask.SendTextToTerminal(TerminalId, Text).await;
				});
			},

			"terminal.dispose" => {
				let ProviderForTask = Provider.clone();

				tokio::spawn(async move {
					let _ = ProviderForTask.DisposeTerminal(TerminalId).await;
				});
			},

			_ => {},
		}
	}
}
