//! Starts a debug session: resolves the configuration and adapter descriptor
//! via reverse-RPC to Cocoon, spawns/connects the debug adapter for the
//! descriptor type, registers the session in `ApplicationState`, and notifies
//! Cocoon (`$onDidStartDebugSession`) plus Sky (`sky://debug/sessionStart`).

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
};
use serde_json::{Value, json};
use tauri::Emitter;
use url::Url;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) async fn Fn(
	Environment:&MountainEnvironment,

	_FolderURI:Option<Url>,

	Configuration:Value,
) -> Result<String, CommonError> {
	let SessionID = uuid::Uuid::new_v4().to_string();

	dev_log!(
		"exthost",
		"[DebugProvider] Starting debug session '{}' with config: {:?}",
		SessionID,
		Configuration
	);

	let IPCProvider:Arc<dyn IPCProvider> = Environment.Require();

	let DebugType = Configuration
		.get("type")
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
	let TargetSideCar = Environment
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
			let (Sender, Pid) =
				super::SpawnExecutableAdapter::Fn(Environment, &Descriptor, &SessionID, &TargetSideCar).await?;

			AdapterStdinSender = Some(Sender);

			AdapterChildPid = Pid;
		},

		"server" => {
			let Sender = super::ConnectServerAdapter::Fn(Environment, &Descriptor, &SessionID, &TargetSideCar).await?;

			AdapterStdinSender = Some(Sender);

			AdapterChildPid = None;
		},

		"pipeServer" => {
			let Sender =
				super::ConnectPipeServerAdapter::Fn(Environment, &Descriptor, &SessionID, &TargetSideCar).await?;

			AdapterStdinSender = Some(Sender);

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
				"warn: [DebugProvider] Unknown adapter descriptor type '{}' for session '{}' - registering session \
				 without spawn.",
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
	if let Err(RegError) = Environment.ApplicationState.Feature.Debug.RegisterDebugSession(
		crate::ApplicationState::State::FeatureState::Debug::DebugState::DebugSessionEntry {
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
	let _ = Environment.ApplicationHandle.emit(
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
