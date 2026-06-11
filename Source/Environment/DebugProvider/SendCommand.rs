//! Sends a DAP command to a debug session's adapter: frames it onto the
//! adapter's stdin pipe when one is live, otherwise routes it via reverse-RPC
//! (`$sendDAPRequest`) into the owning sidecar.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
};
use serde_json::{Value, json};

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) async fn Fn(
	Environment:&MountainEnvironment,

	SessionID:String,

	Command:String,

	Arguments:Value,
) -> Result<Value, CommonError> {
	dev_log!(
		"exthost",
		"[DebugProvider] SendCommand for session '{}' (command: '{}', args: {:?})",
		SessionID,
		Command,
		Arguments
	);

	// Resolve the active session. Missing entries fall through to the
	// reverse-RPC path below so commands targeting an inline-impl
	// adapter (DebugAdapterInlineImplementation - JS-only adapters
	// running inside Cocoon) still reach their handler.
	let SessionEntry = Environment.ApplicationState.Feature.Debug.GetDebugSession(&SessionID);

	// DAP framing: producer must wrap the JSON message in a
	// `Content-Length: <n>\r\n\r\n<body>` header. Sequence numbers
	// are caller-allocated (the workbench's `RawDebugSession` keeps
	// its own `_currentReqId`); we don't reorder. Wire the request
	// shape that VS Code's `mainThreadDebugService.ts` produces:
	// `{ seq, type: "request", command, arguments }`. Mountain
	// doesn't currently track per-session seq numbers - upstream
	// VS Code increments request_seq on the WORKBENCH side and we
	// just forward verbatim - so we emit `0` here as a placeholder
	// when the caller hasn't supplied one in `Arguments.seq`.
	let RequestSeq = Arguments.get("seq").and_then(Value::as_u64).unwrap_or(0);

	let RequestArguments = Arguments.get("arguments").cloned().unwrap_or(Arguments.clone());

	let DapRequest = json!({
		"seq": RequestSeq,
		"type": "request",
		"command": Command,
		"arguments": RequestArguments,
	});

	if let Some(Entry) = SessionEntry.as_ref() {
		if let Some(Sender) = Entry.StdinSender.as_ref() {
			let Body = serde_json::to_vec(&DapRequest).map_err(|Error| {
				CommonError::IPCError {
					Description:format!("Failed to serialize DAP request for session {}: {}", SessionID, Error),
				}
			})?;

			let Header = format!("Content-Length: {}\r\n\r\n", Body.len());

			let mut Frame = Vec::with_capacity(Header.len() + Body.len());

			Frame.extend_from_slice(Header.as_bytes());

			Frame.extend_from_slice(&Body);

			Sender.send(Frame).map_err(|Error| {
				CommonError::IPCError {
					Description:format!("Adapter stdin channel for session {} closed: {}", SessionID, Error),
				}
			})?;

			// stdio adapters reply asynchronously through the
			// stdout reader task, which fans the response out via
			// `sky://debug/dap-message`. Returning an ack now lets
			// the workbench's request sequencer continue; the actual
			// response is correlated by `request_seq` on the
			// renderer side.
			return Ok(json!({
				"success": true,
				"sessionId": SessionID,
				"command": Command,
				"transport": "stdio",
			}));
		}
	}

	// No live stdin pipe: route via reverse-RPC into the owning
	// sidecar. This covers (1) sessions created with
	// `DebugAdapterInlineImplementation` where the adapter runs
	// inside the extension host, (2) `server` / `pipeServer`
	// descriptors awaiting their connection wiring, and (3)
	// commands fired before `RegisterDebugSession` has landed
	// (rare race during spawn). The Cocoon-side handler dispatches
	// based on session-id stored in `extHostDebug.ts`'s session map.
	let TargetSidecar = SessionEntry
		.as_ref()
		.map(|E| E.SideCarIdentifier.clone())
		.unwrap_or_else(|| "cocoon-main".to_string());

	let SendDapMethod = format!("{}$sendDAPRequest", ProxyTarget::ExtHostDebug.GetTargetPrefix());

	let IPCProvider:Arc<dyn IPCProvider> = Environment.Require();

	match IPCProvider
		.SendRequestToSideCar(
			TargetSidecar,
			SendDapMethod,
			json!([{ "sessionId": SessionID, "request": DapRequest }]),
			15000,
		)
		.await
	{
		Ok(Response) => Ok(Response),

		Err(Error) => {
			dev_log!(
				"exthost",
				"warn: [DebugProvider] reverse-RPC SendCommand failed for session {}: {:?}",
				SessionID,
				Error
			);

			Err(Error)
		},
	}
}
