//! Stops a debug session: sends a graceful DAP `disconnect` to the adapter,
//! unregisters the session (closing its stdin pipe), notifies Cocoon
//! (`$onDidTerminateDebugSession`), and emits `sky://debug/sessionEnd`.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
};
use serde_json::json;
use tauri::Emitter;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) async fn Fn(Environment:&MountainEnvironment, SessionID:String) -> Result<(), CommonError> {
	dev_log!("exthost", "[DebugProvider] StopDebugging request for session '{}'", SessionID);

	// Try a graceful DAP `disconnect` first so the adapter can flush
	// pending state and let the debuggee detach cleanly. Failures
	// are logged-and-tolerated; the unregister below force-closes
	// the stdin pipe regardless.
	if let Some(Entry) = Environment.ApplicationState.Feature.Debug.GetDebugSession(&SessionID) {
		if let Some(Sender) = Entry.StdinSender.as_ref() {
			let DisconnectRequest = json!({
				"seq": 0,
				"type": "request",
				"command": "disconnect",
				"arguments": { "restart": false, "terminateDebuggee": true },
			});

			if let Ok(Body) = serde_json::to_vec(&DisconnectRequest) {
				let Header = format!("Content-Length: {}\r\n\r\n", Body.len());

				let mut Frame = Vec::with_capacity(Header.len() + Body.len());

				Frame.extend_from_slice(Header.as_bytes());

				Frame.extend_from_slice(&Body);

				let _ = Sender.send(Frame);
			}
		}
	}

	// Drop the entry. The drained `Sender` clone in the in-flight
	// stdin writer task will see the channel close on its next `recv`
	// and shut the adapter's stdin, which most adapters interpret
	// as a graceful disconnect.
	let _ = Environment.ApplicationState.Feature.Debug.UnregisterDebugSession(&SessionID);

	let IPCProvider:Arc<dyn IPCProvider> = Environment.Require();

	let TerminateMethod = format!("{}$onDidTerminateDebugSession", ProxyTarget::ExtHostDebug.GetTargetPrefix());

	if let Err(error) = IPCProvider
		.SendNotificationToSideCar("cocoon-main".to_string(), TerminateMethod, json!([{ "id": SessionID.clone() }]))
		.await
	{
		dev_log!(
			"exthost",
			"warn: [DebugProvider] StopDebugging notification failed for '{}': {:?}",
			SessionID,
			error
		);
	}

	let _ = Environment
		.ApplicationHandle
		.emit("sky://debug/sessionEnd", json!({ "sessionId": SessionID.clone() }));

	Ok(())
}
