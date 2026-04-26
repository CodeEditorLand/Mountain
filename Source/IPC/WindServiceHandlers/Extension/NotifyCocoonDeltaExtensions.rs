#![allow(non_snake_case)]
//! Post-install/uninstall Cocoon notification.
//!
//! `$deltaExtensions` adds or removes the supplied descriptors from
//! Cocoon's extension registry and indexes `activationEvents`, but it
//! does **not** fire those events. The workbench emits
//! `$activateByEvent("*")` exactly once at boot; anything installed
//! later never sees its startup events re-fired. This helper bursts the
//! single always-satisfied activation event (`onStartupFinished`) after
//! delta so a VSIX with that trigger activates without a reload.
//!
//! Fire-and-forget: missing Cocoon (`LAND_SPAWN_COCOON=false`) or a
//! transient RPC failure is logged and swallowed.

use serde_json::{Value, json};

use crate::{Vine, dev_log};

/// Cocoon sidecar identifier; matches
/// `CocoonManagement::COCOON_SIDE_CAR_IDENTIFIER`.
const COCOON_SIDE_CAR_IDENTIFIER:&str = "cocoon-main";

/// Timeout for fire-and-forget `$deltaExtensions` notifications; long
/// enough to survive a busy Cocoon but short enough that install
/// feedback isn't blocked on a stalled extension host.
const COCOON_DELTA_TIMEOUT_MS:u64 = 10_000;

pub fn NotifyCocoonDeltaExtensions(ToAdd:Vec<Value>, ToRemove:Vec<Value>) {
	tokio::spawn(async move {
		let Parameters = json!({
			"toAdd": ToAdd,
			"toRemove": ToRemove,
		});

		match Vine::Client::SendRequest(
			&COCOON_SIDE_CAR_IDENTIFIER.to_string(),
			"$deltaExtensions".to_string(),
			Parameters,
			COCOON_DELTA_TIMEOUT_MS,
		)
		.await
		{
			Ok(Response) => {
				dev_log!("extensions", "$deltaExtensions applied: {}", Response);
			},
			Err(Error) => {
				dev_log!("extensions", "warn: $deltaExtensions failed (non-fatal): {}", Error);
				// Skip the activation burst when delta itself failed.
				return;
			},
		}

		// Only `onStartupFinished` is fired post-delta - the one event
		// guaranteed to be already satisfied by the time user
		// interaction could reach the install handler (lifecycle phase
		// Ready). Firing `"*"` would over-activate lazy extensions.
		for Event in ["onStartupFinished"] {
			let ActivationParameters = json!({ "activationEvent": Event });
			match Vine::Client::SendRequest(
				&COCOON_SIDE_CAR_IDENTIFIER.to_string(),
				"$activateByEvent".to_string(),
				ActivationParameters,
				COCOON_DELTA_TIMEOUT_MS,
			)
			.await
			{
				Ok(Response) => {
					dev_log!("extensions", "$activateByEvent({}) post-delta applied: {}", Event, Response);
				},
				Err(Error) => {
					dev_log!(
						"extensions",
						"warn: $activateByEvent({}) post-delta failed (non-fatal): {}",
						Event,
						Error
					);
				},
			}
		}
	});
}
