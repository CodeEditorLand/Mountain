// File: Mountain/Source/Environment/DebugProvider.rs
// Role: Implements the `DebugService` trait for the `MountainEnvironment`.
// Responsibilities:
//   - Manage the registration of debug configuration providers and adapter
//     factories.
//   - Orchestrate the `startDebugging` flow, which involves:
//     1. Calling back to the extension host to resolve the final debug
//        configuration.
//     2. Calling back to the extension host to get the debug adapter executable
//        details.
//     3. Spawning and managing the debug adapter process.
//     4. Mediating communication between the UI, the extension host, and the
//        debug adapter.

//! # DebugProvider Implementation
//!
//! Implements the `DebugService` trait for the `MountainEnvironment`. This
//! provider manages the entire debugging lifecycle, from configuration to
//! adapter communication.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	Debug::DebugService::DebugService,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
};
use async_trait::async_trait;
use log::{info, warn};
use serde_json::{Value, json};
use url::Url;

use super::MountainEnvironment::MountainEnvironment;

#[async_trait]
impl DebugService for MountainEnvironment {
	async fn RegisterDebugConfigurationProvider(
		&self,

		DebugType:String,

		_ProviderHandle:u32,

		_SideCarIdentifier:String,
	) -> Result<(), CommonError> {
		// TODO: Store this registration in ApplicationState to track which sidecar
		// owns which debug type.
		info!(
			"[DebugProvider] Registering DebugConfigurationProvider for type '{}'",
			DebugType
		);
		Ok(())
	}

	async fn RegisterDebugAdapterDescriptorFactory(
		&self,

		DebugType:String,

		_FactoryHandle:u32,

		_SideCarIdentifier:String,
	) -> Result<(), CommonError> {
		// TODO: Store this registration in ApplicationState.
		info!(
			"[DebugProvider] Registering DebugAdapterDescriptorFactory for type '{}'",
			DebugType
		);
		Ok(())
	}

	async fn StartDebugging(&self, _FolderURI:Option<Url>, Configuration:Value) -> Result<String, CommonError> {
		let SessionID = uuid::Uuid::new_v4().to_string();
		info!(
			"[DebugProvider] Starting debug session '{}' with config: {:?}",
			SessionID, Configuration
		);

		let IPCProvider:Arc<dyn IPCProvider> = self.Require();
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

		// For now, assume the main sidecar handles all debugging.
		let TargetSideCar = "cocoon-main".to_string();

		// 1. Resolve configuration (Reverse-RPC to Cocoon)
		info!("[DebugProvider] Resolving debug configuration...");
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
		info!("[DebugProvider] Creating debug adapter descriptor...");
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
		// This is a complex step involving process management. For now, we log and
		// simulate.
		info!("[DebugProvider] Spawning Debug Adapter based on descriptor: {:?}", Descriptor);
		// A full implementation would:
		// - Parse the `Descriptor` (which could be an executable, a server port, etc.).
		// - Spawn a new OS process or connect to a TCP socket.
		// - Create a new `DebugSession` struct to manage the DAP communication stream.
		// - Store this session in `ApplicationState`.

		info!("[DebugProvider] Debug session '{}' started (simulation).", SessionID);
		Ok(SessionID)
	}

	async fn SendCommand(&self, SessionID:String, Command:String, Arguments:Value) -> Result<Value, CommonError> {
		// TODO:
		// 1. Look up the active `DebugSession` in `ApplicationState` using `SessionID`.
		// 2. Serialize the command and arguments into a DAP message.
		// 3. Write the message to the Debug Adapter's stdin/socket.
		// 4. Await a response from the adapter, deserialize it, and return.
		warn!(
			"[DebugProvider] SendCommand for session '{}' (command: '{}', args: {:?}) is not implemented.",
			SessionID, Command, Arguments
		);
		Err(CommonError::NotImplemented { FeatureName:"DebugService.SendCommand".into() })
	}
}
