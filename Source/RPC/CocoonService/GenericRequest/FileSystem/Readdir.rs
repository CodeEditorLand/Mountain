use serde_json::{Value, json};
use tonic::Response;
use ::Vine::Generated::GenericResponse;

pub async fn Fn(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Path = Params
		.as_str()
		.or_else(|| Params.get("path").and_then(|V| V.as_str()))
		.unwrap_or("");

	match tokio::fs::read_dir(Path).await {
		Ok(mut Entries) => {
			let mut Items:Vec<Value> = Vec::new();

			while let Ok(Some(Entry)) = Entries.next_entry().await {
				match Entry.file_name().to_str() {
					Some(Name) => {
						let IsDir = Entry.file_type().await.map(|T| T.is_dir()).unwrap_or(false);

						Items.push(json!({ "name": Name, "type": match IsDir { true => 2u32, false => 1u32 } }));
					},

					None => {},
				}
			}

			super::OkResponse(RequestId, &Items)
		},

		Err(Error) => super::ErrResponse(RequestId, -32000, format!("fs.listDir: {}", Error)),
	}
}
