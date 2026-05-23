//! Cocoon → Mountain `security.incident` notification.
//! Emitted by `Cocoon/.../Services/SecurityService.ts:284` when the
//! Cocoon-side security policy flags a policy breach (extension violated
//! its declared permission set, blocked filesystem access, etc.). Land
//! has no central security dashboard yet, so the atom surfaces the
//! incident via `dev_log!` on the `grpc` tag and re-emits on
//! `sky://security/incident` for any future Sky listener.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn SecurityIncident(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://security/incident", Parameter);

	dev_log!(
		"grpc",
		"warn: [Security] incident type={} severity={} ext={}",
		Parameter.get("type").and_then(Value::as_str).unwrap_or("?"),
		Parameter.get("severity").and_then(Value::as_str).unwrap_or("?"),
		Parameter.get("extensionId").and_then(Value::as_str).unwrap_or("?")
	);
}
