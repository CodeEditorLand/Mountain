use std::time::UNIX_EPOCH;

use serde_json::{Value, json};

use tonic::Response;

use ::Vine::Generated::GenericResponse;

pub async fn Fn(RequestId:u64, Params:Value) -> Response<GenericResponse> {

	let Path = Params
		.as_str()
		.or_else(|| Params.get("path").and_then(|V| V.as_str()))
		.unwrap_or("");

	match tokio::fs::metadata(Path).await {
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

		Err(Error) => super::ErrResponse(RequestId, -32000, format!("fs.stat: {}", Error)),
	}
}
