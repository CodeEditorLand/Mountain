use std::time::UNIX_EPOCH;

use serde_json::{Value, json};

use tonic::Response;

use ::Vine::Generated::GenericResponse;

pub async fn Fn(RequestId:u64, Params:Value) -> Response<GenericResponse> {

	let Uri = Params
		.get("uri")
		.and_then(|V| V.as_str())
		.or_else(|| Params.as_str())
		.unwrap_or("")
		.replace("file://", "");

	match tokio::fs::metadata(&Uri).await {
		Ok(Meta) => {
			let Mtime = Meta
				.modified()
				.ok()
				.and_then(|T| T.duration_since(UNIX_EPOCH).ok())
				.map(|D| D.as_millis() as u64)
				.unwrap_or(0);

			super::OkResponse(
				RequestId,

				&json!({
					"type": if Meta.is_dir() { 2 } else { 1 },
					"is_file": Meta.is_file(),
					"is_directory": Meta.is_dir(),
					"size": Meta.len(),
					"mtime": Mtime,
				}),
			)
		},

		Err(Error) => super::ErrResponse(RequestId, -32000, format!("stat: {}", Error)),
	}
}
