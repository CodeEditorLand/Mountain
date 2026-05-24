//! # DebugProvider (Environment)
//!
//! Implements [`DebugService`](CommonLibrary::Debug::DebugService) for
//! `MountainEnvironment`, managing the complete debugging session lifecycle
//! from configuration to termination. Orchestrates between the extension host
//! (Cocoon), the debug adapter, and the UI, including DAP (Debug Adapter
//! Protocol) message mediation.
//!
//! Uses two-stage registration: configuration providers and adapter descriptor
//! factories. Each debug type (node, java, rust) can have its own configuration
//! and adapter. Integrates with
//! [`IPCProvider`](CommonLibrary::IPC::IPCProvider) for RPC to Cocoon.
//!
//! ## Debug session flow
//!
//! 1. UI calls `StartDebugging` with folder URI and configuration.
//! 2. Mountain RPCs to Cocoon to resolve debug configuration (variable
//!    substitution).
//! 3. Mountain RPCs to Cocoon to create debug adapter descriptor.
//! 4. Mountain spawns debug adapter process or connects to TCP server.
//! 5. Mountain mediates DAP messages between UI and debug adapter.
//! 6. UI sends DAP commands via `SendCommand` which forwards to adapter.
//! 7. Debug adapter sends DAP events/notifications back through Mountain to UI.
//! 8. Session ends on stop request or adapter process exit.
//!
//! ## Methods
//!
//! - `RegisterDebugConfigurationProvider` - register config resolver
//! - `RegisterDebugAdapterDescriptorFactory` - register adapter factory
//! - `StartDebugging` - start debug session
//! - `SendCommand` - send DAP command to adapter
//! - `StopDebugging` - graceful DAP disconnect then session unregister
//!
//! ## VS Code reference
//!
//! - `vs/workbench/contrib/debug/browser/debugService.ts`
//! - `vs/workbench/contrib/debug/common/debug.ts`
//! - `vs/workbench/contrib/debug/browser/adapter/descriptorFactory.ts`
//! - `vs/debugAdapter/common/debugProtocol.ts`

use std::sync::Arc;

use CommonLibrary::{
	Debug::DebugService::DebugService,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
};
use async_trait::async_trait;
use serde_json::{Value, json};
use tauri::Emitter;
use url::Url;

use super::MountainEnvironment::Struct;
use crate::dev_log;

#[async_trait]
impl DebugService for MountainEnvironment {
	async fn RegisterDebugConfigurationProvider(
		&self,

		DebugType:String,

		ProviderHandle:u32,

		SideCarIdentifier:String,
	) -> Result<(), CommonError> {
		// Validate debug type is non-empty
		if DebugType.is_empty() {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"DebugType".to_string(),
				Reason:"DebugType cannot be empty".to_string(),
			});
		}

		dev_log!(
			"exthost",
			"[DebugProvider] Registering DebugConfigurationProvider for type '{}' (handle: {}, sidecar: {})",
			DebugType,
			ProviderHandle,
			SideCarIdentifier
		);

		// Store debug configuration provider registration in ApplicationState
		This.ApplicationState
			.Feature
			.Debug
			.RegisterDebugConfigurationProvider(DebugType, ProviderHandle, SideCarIdentifier)
			.map_err(|E| CommonError::Unknown { Description:e })?;

		Ok(())
	}

	async fn RegisterDebugAdapterDescriptorFactory(
		&self,

		DebugType:String,

		FactoryHandle:u32,

		SideCarIdentifier:String,
	) -> Result<(), CommonError> {
		// Validate debug type is non-empty
		if DebugType.is_empty() {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"DebugType".to_string(),
				Reason:"DebugType cannot be empty".to_string(),
			});
		}

		dev_log!(
			"exthost",
			"[DebugProvider] Registering DebugAdapterDescriptorFactory for type '{}' (handle: {}, sidecar: {})",
			DebugType,
			FactoryHandle,
			SideCarIdentifier
		);

		// Store debug adapter descriptor factory registration in ApplicationState
		This.ApplicationState
			.Feature
			.Debug
			.RegisterDebugAdapterDescriptorFactory(DebugType, FactoryHandle, SideCarIdentifier)
			.map_err(|E| CommonError::Unknown { Description:e })?;

		Ok(())
	}

	async fn StartDebugging(&self, _FolderURI:Option<Url>, Configuration:Value) -> Result<String, CommonError> {
		let SessionID = uuid::Uuid::new_v4().to_string();

		dev_log!(
			"exthost",
			"[DebugProvider] Starting debug session '{}' with config: {:?}",
			SessionID,
			Configuration
		);

		let IPCProvider:Arc<dyn IPCProvider> = This.Require();

		let DebugType = Configuration
			.Get("type")
			.and_then(Value::as_str)
			.ok_or_else(|| {
				CommonError::InvalidArgument {
					ArgumentName:"Configuration".into(),

					Reason:"Missing 'type' field in debug configuration.".into(),
				}
			})?
			.to_string();

		// Look up the registered debug configuration provider to get the
		// sidecar that handles this debug type. Falls back to "cocoon-main"
		// (the only extension host today; Grove multi-host will need routing).
		let TargetSideCar = self
			.ApplicationState
			.Feature
			.Debug
			.GetDebugConfigurationProvider(&DebugType)
			.map(|R| R.SideCarIdentifier.clone())
			.unwrap_or_else(|| "cocoon-main".to_string());

		// 1. Resolve configuration (Reverse-RPC to Cocoon)
		dev_log!(
			"exthost",
			"[DebugProvider] Resolving debug configuration for type '{}'",
			DebugType
		);

		dev_log!("exthost", "[DebugProvider] Resolving debug configuration...");

		let ResolveConfigMethod = format!("{}$resolveDebugConfiguration", ProxyTarget::ExtHostDebug.GetTargetPrefix());

		let ResolvedConfig = IPCProvider
			.SendRequestToSideCar(
				TargetSideCar.clone(),
				ResolveConfigMethod,
				json!([DebugType.clone(), Configuration]),
				5000,
			)
			.await?;

		// 2. Get the Debug Adapter Descriptor (Reverse-RPC to Cocoon)
		dev_log!("exthost", "[DebugProvider] Creating debug adapter descriptor...");

		let CreateDescriptorMethod =
			format!("{}$createDebugAdapterDescriptor", ProxyTarget::ExtHostDebug.GetTargetPrefix());

		let Descriptor = IPCProvider
			.SendRequestToSideCar(
				TargetSideCar.clone(),
				CreateDescriptorMethod,
				json!([DebugType, &ResolvedConfig]),
				5000,
			)
			.await?;

		// 3. Spawn the Debug Adapter process based on the descriptor.
		dev_log!(
			"exthost",
			"[DebugProvider] Spawning Debug Adapter based on descriptor: {:?}",
			Descriptor
		);

		// Adapter-descriptor DTO shapes mirror VS Code's
		// `vs/workbench/api/common/extHostDebugService.ts::convert*ToDto`:
		//   executable  → { type: "executable", command, args, options: { env?, cwd? }
		// }   server      → { type: "server", port, host? }
		//   pipeServer  → { type: "pipeServer", path }
		//   implementation → { type: "implementation" }   (handled in-process by
		// Cocoon)
		//
		// Phase 1 supports `executable` only - covers every JS/TS debug adapter
		// (vscode-js-debug, node) and most language-server-driven adapters that
		// ship as a CLI binary. Server/pipeServer connections are stubbed with a
		// warn-log + a session-registry entry without a StdinSender, so SendCommand
		// can surface "adapter type unsupported" instead of a silent no-op.
		// TODO: Wire server / pipeServer adapter connection (TCP / named-pipe).
		// TODO: Wire reverse-RPC `$sendDAPRequest` Cocoon handler for inline-impl
		// adapters.
		let DescriptorType = Descriptor.get("type").and_then(Value::as_str).unwrap_or("").to_string();

		let AdapterStdinSender:Option<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>;

		let AdapterChildPid:Option<u32>;

		match DescriptorType.as_str() {
			"executable" => {
				let Command = Descriptor
					.Get("command")
					.and_then(Value::as_str)
					.ok_or_else(|| {
						CommonError::InvalidArgument {
							ArgumentName:"Descriptor.command".into(),
							Reason:"executable adapter descriptor missing 'command'".into(),
						}
					})?
					.to_string();

				let Args:Vec<String> = Descriptor
					.Get("args")
					.and_then(Value::as_array)
					.map(|A| A.iter().filter_map(|V| V.as_str().map(str::to_string)).collect())
					.unwrap_or_default();

				let OptionsValue = Descriptor.get("options").cloned().unwrap_or(Value::Null);

				let Cwd = OptionsValue.get("cwd").and_then(Value::as_str).map(str::to_string);

				let EnvOverrides:Vec<(String, String)> = OptionsValue
					.Get("env")
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
					CommonError::IPCError {
						Description:format!("Adapter for session {} had no stdout pipe", SessionID),
					}
				})?;

				let Stderr = Child.stderr.take().ok_or_else(|| {
					CommonError::IPCError {
						Description:format!("Adapter for session {} had no stderr pipe", SessionID),
					}
				})?;

				let (Sender, mut Receiver) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

				// Stdin writer task: drains the mpsc channel into the
				// adapter's stdin. Closes when the channel's sender is
				// dropped (UnregisterDebugSession) which propagates EOF
				// to the adapter and triggers its shutdown.
				let StdinSessionId = SessionID.clone();

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
				let StdoutSessionId = SessionID.clone();

				let StdoutHandle = This.ApplicationHandle.clone();

				let StdoutSidecar = TargetSideCar.clone();

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
				let StderrSessionId = SessionID.clone();

				tokio::spawn(async move {
					use tokio::io::{AsyncBufReadExt, BufReader};
					let mut Lines = BufReader::new(Stderr).lines();
					while let Ok(Some(Line)) = Lines.next_line().await {
						crate::dev_log!("exthost", "[DebugAdapter] stderr session={}: {}", StderrSessionId, Line);
					}
				});

				AdapterStdinSender = Some(Sender);

				AdapterChildPid = Pid;

				dev_log!(
					"exthost",
					"[DebugProvider] Spawned executable adapter for session '{}' pid={:?} command={:?}",
					SessionID,
					Pid,
					Command
				);
			},

			"server" | "pipeServer" => {
				dev_log!(
					"exthost",
					"warn: [DebugProvider] Adapter type '{}' not yet wired (session '{}'). Reverse-RPC dispatch only.",
					DescriptorType,
					SessionID
				);

				AdapterStdinSender = None;

				AdapterChildPid = None;
			},

			"implementation" => {
				dev_log!(
					"exthost",
					"[DebugProvider] Inline implementation adapter for session '{}' - DAP frames travel via Cocoon \
					 reverse-RPC.",
					SessionID
				);

				AdapterStdinSender = None;

				AdapterChildPid = None;
			},

			_ => {
				dev_log!(
					"exthost",
					"warn: [DebugProvider] Unknown adapter descriptor type '{}' for session '{}' - registering \
					 session without spawn.",
					DescriptorType,
					SessionID
				);

				AdapterStdinSender = None;

				AdapterChildPid = None;
			},
		}

		// Persist the session in ApplicationState so SendCommand can
		// resolve it. Without this, every subsequent DAP command from the
		// workbench would land on the "session not found" path even though
		// the adapter is alive and listening.
		if let Err(RegError) = This.ApplicationState.Feature.Debug.RegisterDebugSession(
			crate::ApplicationState::Struct::FeatureState::Debug::DebugState::DebugSessionEntry {
				SessionId:SessionID.clone(),
				DebugType:DebugType.clone(),
				SideCarIdentifier:TargetSideCar.clone(),
				StdinSender:AdapterStdinSender,
				ChildPid:AdapterChildPid,
			},
		) {
			dev_log!(
				"exthost",
				"warn: [DebugProvider] Failed to register session '{}' in DebugState: {}",
				SessionID,
				RegError
			);
		}

		// Notify Cocoon that the session has started so any
		// `vscode.debug.onDidStartDebugSession` listeners (registered
		// from extensions through `DebugNamespace.ts:124`) fire. The
		// payload mirrors VS Code's wire shape - extensions read
		// `id`, `type`, `name`, and `configuration` off the session
		// object passed to the listener. Until full session tracking
		// lands in ApplicationState we synthesise from the resolved
		// configuration so extensions can observe activation even
		// while the adapter spawn path is still a stub.
		let StartedMethod = format!("{}$onDidStartDebugSession", ProxyTarget::ExtHostDebug.GetTargetPrefix());

		let StartedSession = json!({
			"id": SessionID.clone(),
			"type": DebugType.clone(),
			"name": ResolvedConfig.get("name").and_then(Value::as_str).unwrap_or(&DebugType),
			"configuration": ResolvedConfig.clone(),
		});

		if let Err(error) = IPCProvider
			.SendNotificationToSideCar(TargetSideCar.clone(), StartedMethod, json!([StartedSession]))
			.await
		{
			dev_log!(
				"exthost",
				"warn: [DebugProvider] StartDebugging notification failed for '{}': {:?}",
				SessionID,
				error
			);
		}

		// Sky-side debug viewlet observers consume this stream so the
		// debug toolbar / call stack panel light up without waiting on
		// the typed `DebugService::ActiveSessions` snapshot. Mirrors
		// `WebviewLifecycle.rs`'s pattern of dual-emitting to Cocoon
		// (typed RPC) and Sky (renderer event).
		let _ = This.ApplicationHandle.emit(
			"sky://debug/sessionStart",
			json!({
				"sessionId": SessionID.clone(),
				"type": DebugType.clone(),
				"configuration": ResolvedConfig.clone(),
			}),
		);

		dev_log!("exthost", "[DebugProvider] Debug session '{}' started (simulation).", SessionID);

		Ok(SessionID)
	}

	async fn SendCommand(&self, SessionID:String, Command:String, Arguments:Value) -> Result<Value, CommonError> {
		dev_log!(
			"exthost",
			"[DebugProvider] SendCommand for session '{}' (command: '{}', args: {:?})",
			SessionID,
			Command,
			Arguments
		);

		// Resolve the active session. Missing entries fall through to the
		// reverse-RPC path below so commands targeting an inline-impl
		// adapter (DebugAdapterInlineImplementation - JS-only adapters
		// running inside Cocoon) still reach their handler.
		let SessionEntry = This.ApplicationState.Feature.Debug.GetDebugSession(&SessionID);

		// DAP framing: producer must wrap the JSON message in a
		// `Content-Length: <n>\r\n\r\n<body>` header. Sequence numbers
		// are caller-allocated (the workbench's `RawDebugSession` keeps
		// its own `_currentReqId`); we don't reorder. Wire the request
		// shape that VS Code's `mainThreadDebugService.ts` produces:
		// `{ seq, type: "request", command, arguments }`. Mountain
		// doesn't currently track per-session seq numbers - upstream
		// VS Code increments request_seq on the WORKBENCH side and we
		// just forward verbatim - so we emit `0` here as a placeholder
		// when the caller hasn't supplied one in `Arguments.seq`.
		let RequestSeq = Arguments.get("seq").and_then(Value::as_u64).unwrap_or(0);

		let RequestArguments = Arguments.get("arguments").cloned().unwrap_or(Arguments.clone());

		let DapRequest = json!({
			"seq": RequestSeq,
			"type": "request",
			"command": Command,
			"arguments": RequestArguments,
		});

		if let Some(Entry) = SessionEntry.as_ref() {
			if let Some(Sender) = Entry.StdinSender.as_ref() {
				let Body = serde_json::to_vec(&DapRequest).map_err(|Error| {
					CommonError::IPCError {
						Description:format!("Failed to serialize DAP request for session {}: {}", SessionID, Error),
					}
				})?;

				let Header = format!("Content-Length: {}\r\n\r\n", Body.len());

				let mut Frame = Vec::with_capacity(Header.len() + Body.len());

				Frame.extend_from_slice(Header.as_bytes());

				Frame.extend_from_slice(&Body);

				Sender.send(Frame).map_err(|Error| {
					CommonError::IPCError {
						Description:format!("Adapter stdin channel for session {} closed: {}", SessionID, Error),
					}
				})?;

				// stdio adapters reply asynchronously through the
				// stdout reader task, which fans the response out via
				// `sky://debug/dap-message`. Returning an ack now lets
				// the workbench's request sequencer continue; the actual
				// response is correlated by `request_seq` on the
				// renderer side.
				return Ok(json!({
					"success": true,
					"sessionId": SessionID,
					"command": Command,
					"transport": "stdio",
				}));
			}
		}

		// No live stdin pipe: route via reverse-RPC into the owning
		// sidecar. This covers (1) sessions created with
		// `DebugAdapterInlineImplementation` where the adapter runs
		// inside the extension host, (2) `server` / `pipeServer`
		// descriptors awaiting their connection wiring, and (3)
		// commands fired before `RegisterDebugSession` has landed
		// (rare race during spawn). The Cocoon-side handler dispatches
		// based on session-id stored in `extHostDebug.ts`'s session map.
		let TargetSidecar = SessionEntry
			.as_ref()
			.map(|E| E.SideCarIdentifier.clone())
			.unwrap_or_else(|| "cocoon-main".to_string());

		let SendDapMethod = format!("{}$sendDAPRequest", ProxyTarget::ExtHostDebug.GetTargetPrefix());

		let IPCProvider:Arc<dyn IPCProvider> = This.Require();

		match IPCProvider
			.SendRequestToSideCar(
				TargetSidecar,
				SendDapMethod,
				json!([{ "sessionId": SessionID, "request": DapRequest }]),
				15000,
			)
			.await
		{
			Ok(Response) => Ok(Response),

			Err(Error) => {
				dev_log!(
					"exthost",
					"warn: [DebugProvider] reverse-RPC SendCommand failed for session {}: {:?}",
					SessionID,
					Error
				);

				Err(Error)
			},
		}
	}

	async fn StopDebugging(&self, SessionID:String) -> Result<(), CommonError> {
		dev_log!("exthost", "[DebugProvider] StopDebugging request for session '{}'", SessionID);

		// Try a graceful DAP `disconnect` first so the adapter can flush
		// pending state and let the debuggee detach cleanly. Failures
		// are logged-and-tolerated; the unregister below force-closes
		// the stdin pipe regardless.
		if let Some(Entry) = This.ApplicationState.Feature.Debug.GetDebugSession(&SessionID) {
			if let Some(Sender) = Entry.StdinSender.as_ref() {
				let DisconnectRequest = json!({
					"seq": 0,
					"type": "request",
					"command": "disconnect",
					"arguments": { "restart": false, "terminateDebuggee": true },
				});

				if let Ok(Body) = serde_json::to_vec(&DisconnectRequest) {
					let Header = format!("Content-Length: {}\r\n\r\n", Body.len());

					let mut Frame = Vec::with_capacity(Header.len() + Body.len());

					Frame.extend_from_slice(Header.as_bytes());

					Frame.extend_from_slice(&Body);

					let _ = Sender.send(Frame);
				}
			}
		}

		// Drop the entry. The drained `Sender` clone in the in-flight
		// stdin writer task will see the channel close on its next `recv`
		// and shut the adapter's stdin, which most adapters interpret
		// as a graceful disconnect.
		let _ = This.ApplicationState.Feature.Debug.UnregisterDebugSession(&SessionID);

		let IPCProvider:Arc<dyn IPCProvider> = This.Require();

		let TerminateMethod = format!("{}$onDidTerminateDebugSession", ProxyTarget::ExtHostDebug.GetTargetPrefix());

		if let Err(error) = IPCProvider
			.SendNotificationToSideCar("cocoon-main".to_string(), TerminateMethod, json!([{ "id": SessionID.clone() }]))
			.await
		{
			dev_log!(
				"exthost",
				"warn: [DebugProvider] StopDebugging notification failed for '{}': {:?}",
				SessionID,
				error
			);
		}

		let _ = self
			.ApplicationHandle
			.emit("sky://debug/sessionEnd", json!({ "sessionId": SessionID.clone() }));

		Ok(())
	}
}
