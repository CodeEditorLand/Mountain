
use serde_json::Value;
use tonic::Response;

use crate::Vine::Generated::GenericResponse;

pub async fn Fn(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Path = Params.get("path").and_then(|V| V.as_str()).unwrap_or("");

	let Content:Vec<u8> = Params
		.get("content")
		.and_then(|V| V.as_array())
		.map(|A| A.iter().filter_map(|B| B.as_u64().map(|N| N as u8)).collect())
		.unwrap_or_default();

	match tokio::fs::write(Path, &Content).await {
		Ok(()) => super::OkResponse(RequestId, &Value::Null),

		Err(Error) => super::ErrResponse(RequestId, -32000, format!("fs.writeFile: {}", Error)),
	}
}
