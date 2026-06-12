//! Spawns an `executable` debug adapter descriptor as a child process and
//! wires its stdio: an mpsc-fed stdin writer task, a stdout DAP-frame reader
//! that re-emits each message on `sky://debug/dap-message`, and a stderr
//! drain that surfaces adapter logs through `dev_log`.

use CommonLibrary::Error::CommonError::CommonError;
use serde_json::{Value, json};
use tauri::Emitter;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) async fn Fn(
	Environment:&MountainEnvironment,

	Descriptor:&Value,

	SessionID:&str,

	TargetSideCar:&str,
) -> Result<(tokio::sync::mpsc::UnboundedSender<Vec<u8>>, Option<u32>), CommonError> {
	let Command = Descriptor
		.get("command")
		.and_then(Value::as_str)
		.ok_or_else(|| {
			CommonError::InvalidArgument {
				ArgumentName:"Descriptor.command".into(),
				Reason:"executable adapter descriptor missing 'command'".into(),
			}
		})?
		.to_string();

	let Args:Vec<String> = Descriptor
		.get("args")
		.and_then(Value::as_array)
		.map(|A| A.iter().filter_map(|V| V.as_str().map(str::to_string)).collect())
		.unwrap_or_default();

	let OptionsValue = Descriptor.get("options").cloned().unwrap_or(Value::Null);

	let Cwd = OptionsValue.get("cwd").and_then(Value::as_str).map(str::to_string);

	let EnvOverrides:Vec<(String, String)> = OptionsValue
		.get("env")
		.and_then(Value::as_object)
		.map(|O| {
			O.iter()
				.filter_map(|(K, V)| V.as_str().map(|S| (K.clone(), S.to_string())))
				.collect()
		})
		.unwrap_or_default();

	let mut Builder = tokio::process::Command::new(&Command);

	Builder
		.args(&Args)
		.stdin(std::process::Stdio::piped())
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped());

	if let Some(CwdPath) = &Cwd {
		Builder.current_dir(CwdPath);
	}

	for (Key, Value) in &EnvOverrides {
		Builder.env(Key, Value);
	}

	let mut Child = Builder.spawn().map_err(|Error| {
		CommonError::IPCError {
			Description:format!(
				"Failed to spawn debug adapter '{}' for session {}: {}",
				Command, SessionID, Error
			),
		}
	})?;

	let Pid = Child.id();

	let Stdin = Child.stdin.take().ok_or_else(|| {
		CommonError::IPCError { Description:format!("Adapter for session {} had no stdin pipe", SessionID) }
	})?;

	let Stdout = Child.stdout.take().ok_or_else(|| {
		CommonError::IPCError { Description:format!("Adapter for session {} had no stdout pipe", SessionID) }
	})?;

	let Stderr = Child.stderr.take().ok_or_else(|| {
		CommonError::IPCError { Description:format!("Adapter for session {} had no stderr pipe", SessionID) }
	})?;

	let (Sender, mut Receiver) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

	// Stdin writer task: drains the mpsc channel into the
	// adapter's stdin. Closes when the channel's sender is
	// dropped (UnregisterDebugSession) which propagates EOF
	// to the adapter and triggers its shutdown.
	let StdinSessionId = SessionID.to_string();

	tokio::spawn(async move {
		use tokio::io::AsyncWriteExt;

		let mut Pipe = Stdin;

		while let Some(Frame) = Receiver.recv().await {
			if let Err(Error) = Pipe.write_all(&Frame).await {
				crate::dev_log!(
					"exthost",
					"warn: [DebugAdapter] stdin write failed for session {}: {}",
					StdinSessionId,
					Error
				);

				break;
			}

			if let Err(Error) = Pipe.flush().await {
				crate::dev_log!(
					"exthost",
					"warn: [DebugAdapter] stdin flush failed for session {}: {}",
					StdinSessionId,
					Error
				);

				break;
			}
		}

		let _ = Pipe.shutdown().await;
	});

	// Stdout reader task: parses DAP frames
	// (`Content-Length: <n>\r\n\r\n<json>`) and re-emits each
	// JSON message on `sky://debug/dap-message` so the
	// renderer / Cocoon-side reverse-RPC can route it to the
	// originating session listener. Errors break the read
	// loop and trigger session cleanup.
	let StdoutSessionId = SessionID.to_string();

	let StdoutHandle = Environment.ApplicationHandle.clone();

	let StdoutSidecar = TargetSideCar.to_string();

	tokio::spawn(async move {
		use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

		let mut Reader = BufReader::new(Stdout);

		let mut Header = String::new();

		loop {
			Header.clear();

			let mut ContentLength:usize = 0;

			loop {
				Header.clear();

				match Reader.read_line(&mut Header).await {
					Ok(0) => return, // EOF
					Ok(_) => {},
					Err(Error) => {
						crate::dev_log!(
							"exthost",
							"warn: [DebugAdapter] stdout read failed for session {}: {}",
							StdoutSessionId,
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
					"warn: [DebugAdapter] stdout body read failed for session {}: {}",
					StdoutSessionId,
					Error
				);

				return;
			}

			let Parsed:Value = serde_json::from_slice(&Body).unwrap_or(Value::Null);

			let _ = StdoutHandle.emit(
				"sky://debug/dap-message",
				json!({
					"sessionId": StdoutSessionId,
					"sidecarId": StdoutSidecar,
					"message": Parsed,
				}),
			);
		}
	});

	// Stderr drain: emit each line as a dev_log line so adapter
	// crash reasons surface alongside other Mountain logs.
	let StderrSessionId = SessionID.to_string();

	tokio::spawn(async move {
		use tokio::io::{AsyncBufReadExt, BufReader};

		let mut Lines = BufReader::new(Stderr).lines();

		while let Ok(Some(Line)) = Lines.next_line().await {
			crate::dev_log!("exthost", "[DebugAdapter] stderr session={}: {}", StderrSessionId, Line);
		}
	});

	dev_log!(
		"exthost",
		"[DebugProvider] Spawned executable adapter for session '{}' pid={:?} command={:?}",
		SessionID,
		Pid,
		Command
	);

	Ok((Sender, Pid))
}
