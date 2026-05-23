//! Wire method: `cocoon:extensionHostMessage`.
//! Relays binary extension-host protocol messages from Wind/Sky to Cocoon via
//! gRPC GenericNotification. Fire-and-forget - the extension host protocol is
//! fully async; Mountain does not await a reply.

use serde_json::Value;
use tauri::AppHandle;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_val;

pub async fn Fn(_ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ByteCount = Arguments
		.first()
		.map(|P| P.get("data").and_then(|D| D.as_array()).map(|A| A.len()).unwrap_or(0))
		.unwrap_or(0);

	crate::dev_log!("exthost", "cocoon:extensionHostMessage bytes={}", ByteCount);

	let Payload = arg_val(&Arguments, 0);

	tokio::spawn(async move {
		if let Err(Error) = crate::Vine::Client::SendNotification::Fn(
			"cocoon-main".to_string(),
			"extensionHostMessage".to_string(),
			Payload,
		)
		.await
		{
			crate::dev_log!("exthost", "cocoon:extensionHostMessage forward failed: {}", Error);
		}
	});

	Ok(Value::Null)
}
