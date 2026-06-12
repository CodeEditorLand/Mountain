//! Forward a webview→extension message to Sky on
//! `sky://webview/post-message`. The protobuf `oneof` is normalised to
//! a JSON value (string or bytes).
use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use ::Vine::Generated::{Empty, OnDidReceiveMessageRequest, on_did_receive_message_request};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(Service:&CocoonServiceImpl, Request:OnDidReceiveMessageRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] on_did_receive_message: handle={}", Request.handle);

	let Payload = match &Request.message {
		Some(on_did_receive_message_request::Message::StringMessage(S)) => json!(S),

		Some(on_did_receive_message_request::Message::BytesMessage(B)) => json!(B),

		None => serde_json::Value::Null,
	};

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://webview/post-message",
		json!({ "handle": Request.handle, "message": Payload }),
	);

	Ok(Response::new(Empty {}))
}
