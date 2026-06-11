//! Connects to a `server` debug adapter descriptor over TCP and wires the
//! stream: an mpsc-fed writer task plus a DAP-frame reader that re-emits
//! each message on `sky://debug/dap-message`.

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
	// Connect to an already-running debug adapter over TCP. The
	// descriptor shape is `{ type: "server", port, host? }` where
	// `host` defaults to 127.0.0.1. Python debugpy, Go dlv, and
	// most language-server-driven adapters ship in this mode.
	let Port = Descriptor.get("port").and_then(Value::as_u64).ok_or_else(|| {
		CommonError::InvalidArgument {
			ArgumentName:"Descriptor.port".into(),
			Reason:"server adapter descriptor missing 'port'".into(),
		}
	})? as u16;

	let Host = Descriptor
		.get("host")
		.and_then(Value::as_str)
		.unwrap_or("127.0.0.1")
		.to_string();

	let Addr = format!("{}:{}", Host, Port);

	dev_log!(
		"exthost",
		"[DebugProvider] Connecting to debug adapter server at {} (session '{}')",
		Addr,
		SessionID
	);

	let TcpStream = tokio::net::TcpStream::connect(&Addr).await.map_err(|Error| {
		CommonError::IPCError {
			Description:format!(
				"Failed to connect to debug adapter server at {} for session {}: {}",
				Addr, SessionID, Error
			),
		}
	})?;

	let (ReadHalf, WriteHalf) = tokio::io::split(TcpStream);

	let (Sender, mut Receiver) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

	// Writer task: drain mpsc into the TCP write half.
	let WriterSessionId = SessionID.to_string();

	tokio::spawn(async move {
		use tokio::io::AsyncWriteExt;

		let mut Pipe = WriteHalf;

		while let Some(Frame) = Receiver.recv().await {
			if let Err(Error) = Pipe.write_all(&Frame).await {
				crate::dev_log!(
					"exthost",
					"warn: [DebugAdapter/server] write failed for session {}: {}",
					WriterSessionId,
					Error
				);

				break;
			}

			let _ = Pipe.flush().await;
		}
	});

	// Reader task: parse DAP frames from the TCP read half and
	// re-emit each JSON message to Sky.
	let ReaderSessionId = SessionID.to_string();

	let ReaderHandle = Environment.ApplicationHandle.clone();

	let ReaderSidecar = TargetSideCar.to_string();

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
							"warn: [DebugAdapter/server] read failed for session {}: {}",
							ReaderSessionId,
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
					"warn: [DebugAdapter/server] body read failed for session {}: {}",
					ReaderSessionId,
					Error
				);

				return;
			}

			let Parsed:Value = serde_json::from_slice(&Body).unwrap_or(Value::Null);

			let _ = ReaderHandle.emit(
				"sky://debug/dap-message",
				json!({
					"sessionId": ReaderSessionId,
					"sidecarId": ReaderSidecar,
					"message": Parsed,
				}),
			);
		}
	});

	dev_log!(
		"exthost",
		"[DebugProvider] Connected to server adapter at {} for session '{}'",
		Addr,
		SessionID
	);

	Ok(Sender)
}
