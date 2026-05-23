use serde_json::Value;
use tonic::Response;

use crate::Vine::Generated::GenericResponse;

pub async fn Fn(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let From = Params.get("from").and_then(|V| V.as_str()).unwrap_or("");

	let To = Params.get("to").and_then(|V| V.as_str()).unwrap_or("");

	match tokio::fs::rename(From, To).await {
		Ok(()) => super::OkResponse(RequestId, &Value::Null),

		Err(Error) => super::ErrResponse(RequestId, -32000, format!("fs.rename: {}", Error)),
	}
}
