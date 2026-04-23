#![allow(non_snake_case)]
//! Cocoon → Mountain `openExternal` notification.
//! Emitted by `Cocoon/.../APIFactoryService.ts:393` when an extension
//! calls `vscode.env.openExternal(uri)`. Delegates to the platform's
//! default handler via the `opener` crate (already a Mountain dep via
//! `nativeHost:openExternal`). Fire-and-forget; success/failure is
//! logged but not surfaced back to the extension.

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OpenExternal(_Service:&MountainVinegRPCService, Parameter:&Value) {
	let Uri = Parameter.get("uri").and_then(Value::as_str).unwrap_or("");
	if Uri.is_empty() {
		dev_log!("grpc", "[OpenExternal] skip: missing uri");
		return;
	}
	match open::that(Uri) {
		Ok(()) => dev_log!("grpc", "[OpenExternal] uri={} ok", Uri),
		Err(Error) => dev_log!("grpc", "[OpenExternal] uri={} err={}", Uri, Error),
	}
}
