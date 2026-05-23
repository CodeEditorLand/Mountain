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

	match tokio::fs::read_dir(&Uri).await {
		Ok(mut Entries) => {
			let mut Names:Vec<String> = Vec::new();

			while let Ok(Some(Entry)) = Entries.next_entry().await {
				if let Some(Name) = Entry.file_name().to_str() {
					Names.push(Name.to_string());
				}
			}

			super::OkResponse(RequestId, &Names)
		},

		Err(Error) => super::ErrResponse(RequestId, -32000, format!("readdir: {}", Error)),
	}
}
