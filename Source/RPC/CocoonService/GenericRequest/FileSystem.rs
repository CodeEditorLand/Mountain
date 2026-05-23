#![allow(unused_variables, dead_code, unused_imports)]

//! Generic-request file-system handlers for `process_mountain_request`.
//! Handles `fs.*` / `file:*` / `readFile` / `writeFile` / `stat` / `readdir`
//! aliases used by Cocoon's `FileSystemService` and `MountainGRPCClient`.

use std::time::UNIX_EPOCH;

use serde_json::{Value, json};
use tonic::Response;

use crate::Vine::Generated::{GenericResponse, RpcError};

/// Build a successful `GenericResponse` with JSON-serialised value.
pub fn OkResponse(RequestId:u64, Value:&impl serde::Serialize) -> Response<GenericResponse> {
	let Bytes = serde_json::to_vec(Value).unwrap_or_default();

	Response::new(GenericResponse { request_identifier:RequestId, result:Bytes, error:None })
}

/// Build an error `GenericResponse`.
pub fn ErrResponse(RequestId:u64, Code:i32, Message:String) -> Response<GenericResponse> {
	Response::new(GenericResponse {
		request_identifier:RequestId,
		result:Vec::new(),
		error:Some(RpcError { code:Code, message:Message, data:Vec::new() }),
	})
}

pub async fn HandleReadFile(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Path = Params
		.as_str()
		.or_else(|| Params.get("path").and_then(|V| V.as_str()))
		.unwrap_or("");

	match tokio::fs::read(Path).await {
		Ok(Content) => OkResponse(RequestId, &Content),

		Err(Error) => ErrResponse(RequestId, -32000, format!("fs.readFile: {}", Error)),
	}
}

pub async fn HandleReadFileUri(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Uri = Params
		.get("uri")
		.and_then(|V| V.as_str())
		.or_else(|| Params.as_str())
		.unwrap_or("")
		.replace("file://", "");

	match tokio::fs::read(&Uri).await {
		Ok(Content) => OkResponse(RequestId, &Content),

		Err(Error) => ErrResponse(RequestId, -32000, format!("readFile: {}", Error)),
	}
}

pub async fn HandleWriteFile(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Path = Params.get("path").and_then(|V| V.as_str()).unwrap_or("");

	let Content:Vec<u8> = Params
		.get("content")
		.and_then(|V| V.as_array())
		.map(|A| A.iter().filter_map(|B| B.as_u64().map(|N| N as u8)).collect())
		.unwrap_or_default();

	match tokio::fs::write(Path, &Content).await {
		Ok(()) => OkResponse(RequestId, &Value::Null),

		Err(Error) => ErrResponse(RequestId, -32000, format!("fs.writeFile: {}", Error)),
	}
}

pub async fn HandleWriteFileUri(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Uri = Params.get("uri").and_then(|V| V.as_str()).unwrap_or("").replace("file://", "");

	let Content:Vec<u8> = Params
		.get("content")
		.and_then(|V| V.as_array())
		.map(|A| A.iter().filter_map(|B| B.as_u64().map(|N| N as u8)).collect())
		.unwrap_or_default();

	match tokio::fs::write(&Uri, &Content).await {
		Ok(()) => OkResponse(RequestId, &Value::Null),

		Err(Error) => ErrResponse(RequestId, -32000, format!("writeFile: {}", Error)),
	}
}

pub async fn HandleStat(RequestId:u64, Params:Value) -> Response<GenericResponse> {
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

			OkResponse(
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

		Err(Error) => ErrResponse(RequestId, -32000, format!("fs.stat: {}", Error)),
	}
}

pub async fn HandleStatUri(RequestId:u64, Params:Value) -> Response<GenericResponse> {
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

			OkResponse(
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

		Err(Error) => ErrResponse(RequestId, -32000, format!("stat: {}", Error)),
	}
}

pub async fn HandleReaddir(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Path = Params
		.as_str()
		.or_else(|| Params.get("path").and_then(|V| V.as_str()))
		.unwrap_or("");

	match tokio::fs::read_dir(Path).await {
		Ok(mut Entries) => {
			let mut Items:Vec<Value> = Vec::new();

			while let Ok(Some(Entry)) = Entries.next_entry().await {
				if let Some(Name) = Entry.file_name().to_str() {
					let IsDir = Entry.file_type().await.map(|T| T.is_dir()).unwrap_or(false);

					Items.push(json!({ "name": Name, "type": if IsDir { 2u32 } else { 1u32 } }));
				}
			}

			OkResponse(RequestId, &Items)
		},

		Err(Error) => ErrResponse(RequestId, -32000, format!("fs.listDir: {}", Error)),
	}
}

pub async fn HandleReaddirUri(RequestId:u64, Params:Value) -> Response<GenericResponse> {
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

			OkResponse(RequestId, &Names)
		},

		Err(Error) => ErrResponse(RequestId, -32000, format!("readdir: {}", Error)),
	}
}

pub async fn HandleCreateDir(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let Path = Params
		.as_str()
		.or_else(|| Params.get("path").and_then(|V| V.as_str()))
		.unwrap_or("");

	match tokio::fs::create_dir_all(Path).await {
		Ok(()) => OkResponse(RequestId, &Value::Null),

		Err(Error) => ErrResponse(RequestId, -32000, format!("fs.createDir: {}", Error)),
	}
}

pub async fn HandleDelete(RequestId:u64, Params:Value) -> Response<GenericResponse> {
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
		Ok(()) => OkResponse(RequestId, &Value::Null),

		Err(Error) => ErrResponse(RequestId, -32000, format!("fs.delete: {}", Error)),
	}
}

pub async fn HandleRename(RequestId:u64, Params:Value) -> Response<GenericResponse> {
	let From = Params.get("from").and_then(|V| V.as_str()).unwrap_or("");

	let To = Params.get("to").and_then(|V| V.as_str()).unwrap_or("");

	match tokio::fs::rename(From, To).await {
		Ok(()) => OkResponse(RequestId, &Value::Null),

		Err(Error) => ErrResponse(RequestId, -32000, format!("fs.rename: {}", Error)),
	}
}
