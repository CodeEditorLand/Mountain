use serde_json::Value;
use tonic::Response;
use ::Vine::Generated::GenericResponse;

pub async fn Fn(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Path = Params
		.as_str()
		.or_else(|| Params.get("path").and_then(|V| V.as_str()))
		.unwrap_or("");

	let Result = if std::path::Path::new(Path).is_dir() {
		tokio::fs::remove_dir_all(Path).await
	} else {
		tokio::fs::remove_file(Path).await
	};

	match Result {
		Ok(()) => super::OkResponse(RequestId, &Value::Null),

		Err(Error) => super::ErrResponse(RequestId, -32000, format!("fs.delete: {}", Error)),
	}
}
