//! Wire method: `cocoon:request`.
//! Generic renderer→Cocoon RPC bridge for two-way wire methods that expect
//! a reply (e.g. `webview.resolveView`). Waits up to 5 s for the gRPC
//! handshake before dispatching; allows up to 30 s for the Cocoon reply.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgVal;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	crate::dev_log!("ipc", "cocoon:request method={:?}", Arguments.first());

	let MethodOpt = Arguments.first().and_then(|V| V.as_str()).map(|S| S.to_string());

	match MethodOpt {
		None => Err("cocoon:request requires method string in slot 0".to_string()),

		Some(Method) => {
			let Payload = ArgVal(&Arguments, 1);

			// Boot-race guard: the renderer can dispatch `cocoon:request` before
			// Cocoon's gRPC handshake completes. 5000 ms chosen because the
			// bundled-electron boot trace shows Cocoon's `Successfully connected`
			// lands ~620 log lines after the workbench's first request.
			let _ = crate::Vine::Client::WaitForClientConnection::Fn("cocoon-main", 5000).await;

			crate::Vine::Client::SendRequest::Fn("cocoon-main", Method.clone(), Payload, 30_000)
				.await
				.map_err(|Error| format!("cocoon:request {} failed: {:?}", Method, Error))
		},
	}
}
