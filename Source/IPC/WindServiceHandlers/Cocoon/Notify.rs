#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `cocoon:notify`.
//! Fire-and-forget renderer→Cocoon notification bridge for one-way wire
//! methods (`webview.message`, `webview.dispose`, `webview.viewState`, etc.)
//! where the extension doesn't reply. Returns null immediately; the
//! notification dispatches asynchronously avoiding the 30 s request timeout.

use serde_json::Value;

pub async fn CocoonNotify(Arguments:Vec<Value>) -> Result<Value, String> {
	crate::dev_log!("ipc", "cocoon:notify method={:?}", Arguments.first());
	let MethodOpt = Arguments.first().and_then(|V| V.as_str()).map(|S| S.to_string());
	match MethodOpt {
		None => Err("cocoon:notify requires method string in slot 0".to_string()),
		Some(Method) => {
			let Payload = Arguments.get(1).cloned().unwrap_or(Value::Null);
			if let Err(Error) =
				crate::Vine::Client::SendNotification::Fn("cocoon-main".to_string(), Method.clone(), Payload).await
			{
				crate::dev_log!("ipc", "warn: [cocoon:notify] {} failed: {:?}", Method, Error);
			}
			Ok(Value::Null)
		},
	}
}
