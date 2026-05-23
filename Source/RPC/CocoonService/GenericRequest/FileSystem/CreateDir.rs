#![allow(unused_variables, dead_code, unused_imports)]

use serde_json::Value;
use tonic::Response;

use crate::Vine::Generated::GenericResponse;

pub async fn Fn(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Path = Params
		.as_str()
		.or_else(|| Params.get("path").and_then(|V| V.as_str()))
		.unwrap_or("");

	match tokio::fs::create_dir_all(Path).await {
		Ok(()) => super::OkResponse(RequestId, &Value::Null),

		Err(Error) => super::ErrResponse(RequestId, -32000, format!("fs.createDir: {}", Error)),
	}
}
