#![allow(non_snake_case)]

//! Push a message from the extension into the webview via
//! `sky://webview/post-message`. Canonical kebab-case channel;
//! `sky://webview/postMessage` has been retired.

use serde_json::json;

use tauri::Emitter;

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, PostWebviewMessageRequest, post_webview_message_request},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:PostWebviewMessageRequest) -> Result<Response<Empty>, Status> {

	dev_log!("cocoon", "[CocoonService] post_webview_message: handle={}", Request.handle);

	let Payload = match &Request.message {

		Some(post_webview_message_request::Message::StringMessage(S)) => json!(S),

		Some(post_webview_message_request::Message::BytesMessage(B)) => json!(B),

		None => serde_json::Value::Null,
	};

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://webview/post-message",

		json!({ "handle": Request.handle, "message": Payload }),
	);

	Ok(Response::new(Empty {}))
}
