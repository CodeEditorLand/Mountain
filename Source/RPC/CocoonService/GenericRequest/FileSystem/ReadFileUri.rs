#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use serde_json::Value;
use tonic::Response;

use crate::Vine::Generated::GenericResponse;

pub async fn Fn(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Uri = Params
		.get("uri")
		.and_then(|V| V.as_str())
		.or_else(|| Params.as_str())
		.unwrap_or("")
		.replace("file://", "");

	match tokio::fs::read(&Uri).await {
		Ok(Content) => super::OkResponse(RequestId, &Content),

		Err(Error) => super::ErrResponse(RequestId, -32000, format!("readFile: {}", Error)),
	}
}
