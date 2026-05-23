use serde_json::Value;
use tonic::Response;

use crate::Vine::Generated::GenericResponse;

pub async fn Fn(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Path = Params
		.as_str()
		.or_else(|| Params.get("path").and_then(|V| V.as_str()))
		.unwrap_or("");

	match tokio::fs::read(Path).await {
		Ok(Content) => super::OkResponse(RequestId, &Content),

		Err(Error) => super::ErrResponse(RequestId, -32000, format!("fs.readFile: {}", Error)),
	}
}
