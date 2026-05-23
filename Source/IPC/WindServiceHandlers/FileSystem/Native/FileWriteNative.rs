#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method `file:write` / `file:writeFile`. Accepts either a plain
//! string body or a `{ buffer: number[] | base64 }` VSBuffer. Parent
//! directory is created best-effort. After a successful write, fires
//! `$acceptModelSaved` to Cocoon so `onDidSaveTextDocument` reaches
//! extensions (T1.4 save notification).

use serde_json::{Value, json};

use crate::{IPC::WindServiceHandlers::Utilities::PathExtraction::Fn as extract_path_from_arg, dev_log};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let ResourceArg = Arguments.get(0).ok_or("Missing file path")?;

	// Capture the `external` field (full file:// URI) from the URI object
	// for the $acceptModelSaved notification before we consume the path.
	let ExternalUri = ResourceArg
		.as_object()
		.and_then(|O| O.get("external"))
		.and_then(|V| V.as_str())
		.map(|S| S.to_string());

	let Path = extract_path_from_arg(ResourceArg)?;

	let Content = Arguments.get(1).ok_or("Missing file content")?;

	let Bytes = if let Some(S) = Content.as_str() {
		S.as_bytes().to_vec()
	} else if let Some(Obj) = Content.as_object() {
		if let Some(Buf) = Obj.get("buffer") {
			if let Some(Arr) = Buf.as_array() {
				Arr.iter().filter_map(|V| V.as_u64().map(|N| N as u8)).collect()
			} else if let Some(S) = Buf.as_str() {
				S.as_bytes().to_vec()
			} else {
				return Err("Unsupported buffer format".to_string());
			}
		} else {
			serde_json::to_string(Content).unwrap_or_default().into_bytes()
		}
	} else {
		return Err("File content must be a string or VSBuffer".to_string());
	};

	if let Some(Parent) = std::path::Path::new(&Path).parent() {
		tokio::fs::create_dir_all(Parent).await.ok();
	}

	let Start = std::time::Instant::now();

	tokio::fs::write(&Path, &Bytes)
		.await
		.map_err(|E| format!("Failed to write file: {} (path: {})", E, Path))?;

	let ElapsedMs = Start.elapsed().as_millis();

	dev_log!("vfs", "file:write ok path={} bytes={} ms={}", Path, Bytes.len(), ElapsedMs);

	// T1.4 - notify Cocoon that the model on disk now matches the editor
	// buffer so `onDidSaveTextDocument` fires for subscribed extensions.
	// Build a file:// URI from `external` (preferred) or the path string.
	let FileUri = ExternalUri.unwrap_or_else(|| format!("file://{}", Path));

	tokio::spawn(async move {
		if let Err(Error) = crate::Vine::Client::SendNotification::Fn(
			"cocoon-main".to_string(),
			"$acceptModelSaved".to_string(),
			json!({ "uri": FileUri }),
		)
		.await
		{
			dev_log!("vfs", "warn: [FileWriteNative] $acceptModelSaved notify failed: {:?}", Error);
		}
	});

	// Return mtime/size so VS Code's DiskFileSystemProvider can update its
	// FileStatWithMetadata cache - prevents a spurious "file changed on disk"
	// conflict caused by the pre-write etag being stale after the write.
	match tokio::fs::metadata(&Path).await {
		Ok(Meta) => {
			Ok(crate::IPC::WindServiceHandlers::Utilities::MetadataEncoding::Fn(
				&Meta,
			))
		},

		Err(_) => Ok(Value::Null),
	}
}
