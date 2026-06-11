//! Connects to a `pipeServer` debug adapter descriptor over a Unix domain
//! socket (macOS/Linux) or named pipe (Windows) and wires the stream: an
//! mpsc-fed writer task plus a DAP-frame reader that re-emits each message
//! on `sky://debug/dap-message`.

use CommonLibrary::Error::CommonError::CommonError;
use serde_json::{Value, json};
use tauri::Emitter;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) async fn Fn(
	Environment:&MountainEnvironment,

	Descriptor:&Value,

	SessionID:&str,

	TargetSideCar:&str,
) -> Result<tokio::sync::mpsc::UnboundedSender<Vec<u8>>, CommonError> {
	// Connect to an already-running debug adapter over a Unix
	// domain socket (macOS/Linux) or named pipe (Windows). The
	// descriptor shape is `{ type: "pipeServer", path }`.
	let PipePath = Descriptor
		.get("path")
		.and_then(Value::as_str)
		.ok_or_else(|| {
			CommonError::InvalidArgument {
				ArgumentName:"Descriptor.path".into(),
				Reason:"pipeServer adapter descriptor missing 'path'".into(),
			}
		})?
		.to_string();

	dev_log!(
		"exthost",
		"[DebugProvider] Connecting to debug adapter pipe at '{}' (session '{}')",
		PipePath,
		SessionID
	);

	#[cfg(unix)]
	let (ReadHalf, WriteHalf) = {
		let Stream = tokio::net::UnixStream::connect(&PipePath).await.map_err(|Error| {
			CommonError::IPCError {
				Description:format!(
					"Failed to connect to debug adapter pipe '{}' for session {}: {}",
					PipePath, SessionID, Error
				),
			}
		})?;

		tokio::io::split(Stream)
	};

	#[cfg(windows)]
	let (ReadHalf, WriteHalf) = {
		// On Windows, named pipes use \\.\pipe\<name>. Tokio's
		// NamedPipeClient handles both directions.
		let Stream = tokio::net::windows::named_pipe::ClientOptions::new().open(&PipePath).map_err(|Error| {
			CommonError::IPCError {
				Description:format!(
					"Failed to open named pipe '{}' for session {}: {}",
					PipePath, SessionID, Error
				),
			}
		})?;

		tokio::io::split(Stream)
	};

	let (Sender, mut Receiver) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

	let PipeWriterSessionId = SessionID.to_string();

	tokio::spawn(async move {
		use tokio::io::AsyncWriteExt;

		let mut Pipe = WriteHalf;

		while let Some(Frame) = Receiver.recv().await {
			if let Err(Error) = Pipe.write_all(&Frame).await {
				crate::dev_log!(
					"exthost",
					"warn: [DebugAdapter/pipe] write failed for session {}: {}",
					PipeWriterSessionId,
					Error
				);

				break;
			}

			let _ = Pipe.flush().await;
		}
	});

	let PipeReaderSessionId = SessionID.to_string();

	let PipeReaderHandle = Environment.ApplicationHandle.clone();

	let PipeReaderSidecar = TargetSideCar.to_string();

	tokio::spawn(async move {
		use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

		let mut Reader = BufReader::new(ReadHalf);

		let mut Header = String::new();

		loop {
			Header.clear();

			let mut ContentLength:usize = 0;

			loop {
				Header.clear();

				match Reader.read_line(&mut Header).await {
					Ok(0) => return,
					Ok(_) => {},
					Err(Error) => {
						crate::dev_log!(
							"exthost",
							"warn: [DebugAdapter/pipe] read failed for session {}: {}",
							PipeReaderSessionId,
							Error
						);

						return;
					},
				}

				let Trimmed = Header.trim_end_matches("\r\n").trim_end_matches('\n');

				if Trimmed.is_empty() {
					break;
				}

				if let Some(Rest) = Trimmed.strip_prefix("Content-Length:") {
					if let Ok(N) = Rest.trim().parse::<usize>() {
						ContentLength = N;
					}
				}
			}

			if ContentLength == 0 {
				continue;
			}

			let mut Body = vec![0u8; ContentLength];

			if let Err(Error) = Reader.read_exact(&mut Body).await {
				crate::dev_log!(
					"exthost",
					"warn: [DebugAdapter/pipe] body read failed for session {}: {}",
					PipeReaderSessionId,
					Error
				);

				return;
			}

			let Parsed:Value = serde_json::from_slice(&Body).unwrap_or(Value::Null);

			let _ = PipeReaderHandle.emit(
				"sky://debug/dap-message",
				json!({
					"sessionId": PipeReaderSessionId,
					"sidecarId": PipeReaderSidecar,
					"message": Parsed,
				}),
			);
		}
	});

	dev_log!(
		"exthost",
		"[DebugProvider] Connected to pipe adapter at '{}' for session '{}'",
		PipePath,
		SessionID
	);

	Ok(Sender)
}
