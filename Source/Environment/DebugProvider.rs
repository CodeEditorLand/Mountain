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
//
// NOTE: This is a stub implementation and needs to be fully built out.

//! # DebugProvider Implementation
//!
//! Implements the `DebugService` trait for the `MountainEnvironment`.

use std::sync::Arc;

use Common::{
	Debug::DebugService::DebugService,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::IPCProvider::IPCProvider,
};
use async_trait::async_trait;
use log::info;
use serde_json::{Value, json};
use url::Url;

use super::MountainEnvironment::MountainEnvironment;

#[async_trait]
impl DebugService for MountainEnvironment {
	async fn RegisterDebugConfigurationProvider(
		&self,

		DebugType:String,

		_ProviderHandle:u32,

		_SidecarIdentifier:String,
	) -> Result<(), CommonError> {
		// TODO: Store this registration in ApplicationState
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

		_SidecarIdentifier:String,
	) -> Result<(), CommonError> {
		// TODO: Store this registration in ApplicationState
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

		let DebugType = Configuration.get("type").and_then(Value::as_str).unwrap_or_default();

		// 1. Resolve configuration (Reverse-RPC to Cocoon)
		info!("[DebugProvider] Resolving debug configuration...");

		let ResolvedConfig = IPCProvider
			.SendRequestToSidecar(
				"cocoon-main".into(),
				"$resolveDebugConfiguration".into(),
				json!([DebugType, Configuration]),
				5000,
			)
			.await?;

		// 2. Get the Debug Adapter Descriptor (Reverse-RPC to Cocoon)
		info!("[DebugProvider] Creating debug adapter descriptor...");

		let Descriptor = IPCProvider
			.SendRequestToSidecar(
				"cocoon-main".into(),
				"$createDebugAdapterDescriptor".into(),
				json!([DebugType, &ResolvedConfig]),
				5000,
			)
			.await?;

		// 3. Spawn the Debug Adapter process based on the descriptor
		// This is a complex step involving process management. For now, we log it.
		info!("[DebugProvider] Spawning Debug Adapter based on descriptor: {:?}", Descriptor);

		// Placeholder
		// let da_process = spawn_process(descriptor);

		// 4. Create a DebugSession object to manage communication with the DA and the
		//    UI
		// Store this session in ApplicationState
		info!("[DebugProvider] Debug session '{}' started (simulation).", SessionID);

		Ok(SessionID)
	}

	async fn SendCommand(&self, _SessionID:String, _Command:String, _Arguments:Value) -> Result<Value, CommonError> {
		// TODO: Find the session and forward the command to the Debug Adapter process
		Err(CommonError::NotImplemented { FeatureName:"DebugService.SendCommand".into() })
	}
}
