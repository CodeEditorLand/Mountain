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
//! Each method body lives in its own module under `DebugProvider/`; the
//! single trait `impl` here delegates per method (Rust trait impls cannot
//! be split across blocks).
//!
//! ## VS Code reference
//!
//! - `vs/workbench/contrib/debug/browser/debugService.ts`
//! - `vs/workbench/contrib/debug/common/debug.ts`
//! - `vs/workbench/contrib/debug/browser/adapter/descriptorFactory.ts`
//! - `vs/debugAdapter/common/debugProtocol.ts`

pub mod ConnectPipeServerAdapter;

pub mod ConnectServerAdapter;

pub mod RegisterDebugAdapterDescriptorFactory;

pub mod RegisterDebugConfigurationProvider;

pub mod SendCommand;

pub mod SpawnExecutableAdapter;

pub mod StartDebugging;

pub mod StopDebugging;

use CommonLibrary::{Debug::DebugService::DebugService, Error::CommonError::CommonError};
use async_trait::async_trait;
use serde_json::Value;
use url::Url;

use super::MountainEnvironment::MountainEnvironment;

#[async_trait]
impl DebugService for MountainEnvironment {
	async fn RegisterDebugConfigurationProvider(
		&self,

		DebugType:String,

		ProviderHandle:u32,

		SideCarIdentifier:String,
	) -> Result<(), CommonError> {
		RegisterDebugConfigurationProvider::Fn(self, DebugType, ProviderHandle, SideCarIdentifier).await
	}

	async fn RegisterDebugAdapterDescriptorFactory(
		&self,

		DebugType:String,

		FactoryHandle:u32,

		SideCarIdentifier:String,
	) -> Result<(), CommonError> {
		RegisterDebugAdapterDescriptorFactory::Fn(self, DebugType, FactoryHandle, SideCarIdentifier).await
	}

	async fn StartDebugging(&self, FolderURI:Option<Url>, Configuration:Value) -> Result<String, CommonError> {
		StartDebugging::Fn(self, FolderURI, Configuration).await
	}

	async fn SendCommand(&self, SessionID:String, Command:String, Arguments:Value) -> Result<Value, CommonError> {
		SendCommand::Fn(self, SessionID, Command, Arguments).await
	}

	async fn StopDebugging(&self, SessionID:String) -> Result<(), CommonError> {
		StopDebugging::Fn(self, SessionID).await
	}
}
