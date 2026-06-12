//! `file:write` (fd form) - positional write to a descriptor opened via
//! `file:open` with `{ create: true }`.
//!
//! VS Code's `DiskFileSystemProviderClient.write(fd, pos, data, offset,
//! length)` sends `channel.call('write', [fd, pos, VSBuffer, offset,
//! length])` and expects the number of bytes written back. The VSBuffer
//! arrives JSON-serialised: either `{ buffer: [u8...] }`, a bare array, or
//! a `Uint8Array`-stringified index map (`{"0":104,"1":101,...}`); all
//! three shapes are accepted.
//!
//! Arguments\[0\] = integer fd (from FileOpenFd)
//! Arguments\[1\] = position (byte offset from file start)
//! Arguments\[2\] = data (VSBuffer-serialised)
//! Arguments\[3\] = offset into data
//! Arguments\[4\] = length of the slice to write
//!
//! Same take-out / reinsert discipline on the fd table as FileReadFd:
//! the sync Mutex cannot be held across the await-bearing seek+write.

use serde_json::{Value, json};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use crate::{
	IPC::WindServiceHandlers::{FileSystem::Native::FileOpenFd::GetFdTable, Utilities::JsonValueHelpers::arg_u64},
	dev_log,
};

/// Recover raw bytes from any of the JSON shapes a VSBuffer takes on the
/// wire. Returns None when the value carries no byte payload.
fn ExtractBytes(Data:&Value) -> Option<Vec<u8>> {
	if let Some(Array) = Data.as_array() {
		return Some(Array.iter().filter_map(Value::as_u64).map(|B| B as u8).collect());
	}

	if let Some(Object) = Data.as_object() {
		if let Some(Inner) = Object.get("buffer") {
			return ExtractBytes(Inner);
		}

		if let Some(Inner) = Object.get("data") {
			return ExtractBytes(Inner);
		}

		// Uint8Array stringified as an index map: keys "0".."N-1".
		if !Object.is_empty() && Object.keys().all(|K| K.bytes().all(|B| B.is_ascii_digit())) {
			let mut Bytes = vec![0u8; Object.len()];

			for (Key, Val) in Object {
				let Index:usize = Key.parse().ok()?;

				if Index >= Bytes.len() {
					return None;
				}

				Bytes[Index] = Val.as_u64()? as u8;
			}

			return Some(Bytes);
		}
	}

	None
}

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Fd = arg_u64(&Arguments, 0) as u32;

	let Pos = arg_u64(&Arguments, 1);

	let Bytes = Arguments
		.get(2)
		.and_then(ExtractBytes)
		.ok_or_else(|| format!("file:write fd={}: data argument carries no bytes", Fd))?;

	let Offset = arg_u64(&Arguments, 3) as usize;

	let Length = arg_u64(&Arguments, 4) as usize;

	if Fd == 0 {
		return Err("file:write: missing or zero fd".to_string());
	}

	let End = Offset.saturating_add(Length).min(Bytes.len());

	let Slice = Bytes.get(Offset..End).unwrap_or(&[]);

	let mut File = {
		let mut Table = GetFdTable().Files.lock().unwrap_or_else(|E| E.into_inner());

		Table.remove(&Fd).ok_or_else(|| format!("file:write: unknown fd {}", Fd))?
	};

	let WriteResult = async {
		File.seek(std::io::SeekFrom::Start(Pos))
			.await
			.map_err(|E| format!("file:write fd={} seek({}): {}", Fd, Pos, E))?;

		File.write_all(Slice)
			.await
			.map_err(|E| format!("file:write fd={}: {}", Fd, E))?;

		File.flush().await.map_err(|E| format!("file:write fd={} flush: {}", Fd, E))?;

		Ok::<usize, String>(Slice.len())
	}
	.await;

	{
		let mut Table = GetFdTable().Files.lock().unwrap_or_else(|E| E.into_inner());

		Table.insert(Fd, File);
	}

	let Written = WriteResult?;

	dev_log!("vfs-verbose", "file:write fd={} pos={} wrote={}", Fd, Pos, Written);

	Ok(json!(Written))
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_bare_array() {
		assert_eq!(ExtractBytes(&json!([104, 105])), Some(vec![104, 105]));
	}

	#[test]
	fn test_buffer_wrapper() {
		assert_eq!(ExtractBytes(&json!({ "buffer": [104, 105] })), Some(vec![104, 105]));
	}

	#[test]
	fn test_nested_buffer_wrapper() {
		assert_eq!(
			ExtractBytes(&json!({ "buffer": { "buffer": [104, 105] } })),
			Some(vec![104, 105])
		);
	}

	#[test]
	fn test_index_map() {
		assert_eq!(ExtractBytes(&json!({ "0": 104, "1": 105 })), Some(vec![104, 105]));
	}

	#[test]
	fn test_index_map_out_of_range_returns_none() {
		assert_eq!(ExtractBytes(&json!({ "0": 104, "5": 105 })), None);
	}

	#[test]
	fn test_empty_object_returns_none() {
		assert_eq!(ExtractBytes(&json!({})), None);
	}
}
