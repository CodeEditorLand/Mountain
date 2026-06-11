//! `file:read` (fd form) - positional read from a descriptor opened via
//! `file:open`.
//!
//! VS Code's `DiskFileSystemProviderClient.read(fd, pos, data, offset,
//! length)` sends `channel.call('read', [fd, pos, length])` and expects a
//! `[VSBuffer, bytesRead]` tuple back. Mountain returns
//! `{ buffer: [u8...], bytesRead: N }`; Wind's `TauriMainProcessService`
//! reshapes that into the tuple. This is the read primitive behind EVERY
//! editor file open: `TextFileEditorModelManager.resolve` →
//! `fileService.readFileStream` → buffered fd reads.
//!
//! Arguments\[0\] = integer fd (from FileOpenFd)
//! Arguments\[1\] = position (byte offset from file start)
//! Arguments\[2\] = length (max bytes to read)
//!
//! The fd table stores `tokio::fs::File` behind a sync Mutex, so the file
//! is taken out of the table for the await-bearing seek+read and reinserted
//! afterwards (also on error, so a transient failure does not poison the
//! descriptor). VS Code serialises reads per fd, so the brief absence is
//! not observable in practice; a racing read on the same fd fails fast
//! with "unknown fd" rather than corrupting offsets.

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::{
	IPC::WindServiceHandlers::{FileSystem::Native::FileOpenFd::GetFdTable, Utilities::JsonValueHelpers::arg_u64},
	dev_log,
};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Fd = arg_u64(&Arguments, 0) as u32;

	let Pos = arg_u64(&Arguments, 1);

	let Length = arg_u64(&Arguments, 2) as usize;

	if Fd == 0 {
		return Err("file:read: missing or zero fd".to_string());
	}

	let mut File = {
		let mut Table = GetFdTable().Files.lock().unwrap_or_else(|E| E.into_inner());

		Table.remove(&Fd).ok_or_else(|| format!("file:read: unknown fd {}", Fd))?
	};

	let ReadResult = async {
		File.seek(std::io::SeekFrom::Start(Pos))
			.await
			.map_err(|E| format!("file:read fd={} seek({}): {}", Fd, Pos, E))?;

		let mut Buffer = vec![0u8; Length];

		let mut Total = 0usize;

		while Total < Length {
			let N = File.read(&mut Buffer[Total..]).await.map_err(|E| format!("file:read fd={}: {}", Fd, E))?;

			if N == 0 {
				break;
			}

			Total += N;
		}

		Buffer.truncate(Total);

		Ok::<Vec<u8>, String>(Buffer)
	}
	.await;

	{
		let mut Table = GetFdTable().Files.lock().unwrap_or_else(|E| E.into_inner());

		Table.insert(Fd, File);
	}

	let Buffer = ReadResult?;

	dev_log!("vfs-verbose", "file:read fd={} pos={} len={} read={}", Fd, Pos, Length, Buffer.len());

	Ok(json!({ "buffer": Buffer, "bytesRead": Buffer.len() }))
}
