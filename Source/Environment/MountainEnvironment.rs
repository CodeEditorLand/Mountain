// File: Mountain/Source/Environment/MountainEnvironment.rs

//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # MountainEnvironment
//!
//! Defines the concrete `MountainEnvironment` struct, which serves as the
//! central context and dependency injection container for the `Mountain`
//! application.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

// Import Air service client when Air integration is enabled
#[cfg(feature = "AirIntegration")]
use Air::Vine::Generated::air_service_client::AirServiceClient;

use Common::{
	Command::CommandExecutor::CommandExecutor,
	Configuration::{ConfigurationInspector::ConfigurationInspector, ConfigurationProvider::ConfigurationProvider},
	CustomEditor::CustomEditorProvider::CustomEditorProvider,
	Debug::DebugService::DebugService,
	Diagnostic::DiagnosticManager::DiagnosticManager,
	Document::DocumentProvider::DocumentProvider,
	Environment::{Environment::Environment, Requires::Requires},
	Error::CommonError::CommonError,
	ExtensionManagement::ExtensionManagementService::ExtensionManagementService,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
	IPC::IPCProvider::IPCProvider,
	Keybinding::KeybindingProvider::KeybindingProvider,
	LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	Output::OutputChannelManager::OutputChannelManager,
	Search::SearchProvider::SearchProvider,
	Secret::SecretProvider::SecretProvider,
	SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider,
	StatusBar::StatusBarProvider::StatusBarProvider,
	Storage::StorageProvider::StorageProvider,
	Synchronization::SynchronizationProvider::SynchronizationProvider,
	Terminal::TerminalProvider::TerminalProvider,
	Testing::TestController::TestController,
	TreeView::TreeViewProvider::TreeViewProvider,
	UserInterface::UserInterfaceProvider::UserInterfaceProvider,
	WebView::WebViewProvider::WebViewProvider,
	WorkSpace::{WorkSpaceEditApplier::WorkSpaceEditApplier, WorkSpaceProvider::WorkSpaceProvider},
};
use async_trait::async_trait;
use log::{info, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Wry};

use crate::ApplicationState::ApplicationState::ApplicationState;

/// The concrete `Environment` for the Mountain application.
#[derive(Clone)]
pub struct MountainEnvironment {
	pub ApplicationHandle:AppHandle<Wry>,

	pub ApplicationState:Arc<ApplicationState>,

	/// Optional Air client for cloud-based services.
	/// When provided, providers like SecretProvider and UpdateService can delegate to Air.
	#[cfg(feature = "AirIntegration")]
	pub AirClient:Option<Arc<AirServiceClient<tonic::transport::Channel>>>,
}

impl MountainEnvironment {
	/// Creates a new `MountainEnvironment` instance.
	#[allow(unused_mut)]
	pub fn Create(ApplicationHandle: AppHandle<Wry>) -> Self {
		info!("[MountainEnvironment] New instance created.");

		let ApplicationState = ApplicationHandle.state::<Arc<ApplicationState>>().inner().clone();

		#[cfg(feature = "AirIntegration")]
		{
			Self { ApplicationHandle, ApplicationState, AirClient: None }
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Self { ApplicationHandle, ApplicationState }
		}
	}

	/// Creates a new `MountainEnvironment` instance with an optional Air client.
	/// When AirClient is provided, providers can delegate to Air for cloud-based services.
	#[cfg(feature = "AirIntegration")]
	pub fn CreateWithAir(
		ApplicationHandle: AppHandle<Wry>,
		AirClient: Option<Arc<AirServiceClient<tonic::transport::Channel>>>,
	) -> Self {
		info!(
			"[MountainEnvironment] New instance created with Air client: {}",
			AirClient.is_some()
		);

		let ApplicationState = ApplicationHandle.state::<Arc<ApplicationState>>().inner().clone();

		Self { ApplicationHandle, ApplicationState, AirClient }
	}

	/// Updates the Air client for this environment.
	/// This allows dynamically switching between Air and local services.
	#[cfg(feature = "AirIntegration")]
	pub fn SetAirClient(&mut self, AirClient: Option<Arc<AirServiceClient<tonic::transport::Channel>>>) {
		info!("[MountainEnvironment] Air client updated: {}", AirClient.is_some());

		self.AirClient = AirClient;
	}

	/// Returns whether Air is available and ready.
	#[cfg(feature = "AirIntegration")]
	pub async fn IsAirAvailable(&self) -> bool {
		if let Some(AirClient) = &self.AirClient {
			use tonic::Request;
			use Air::Vine::Generated::air_service_client::air_service_server;

			match AirClient
				.health_check(Request::new(air_service_server::HealthCheckRequest {}))
				.await
			{
				Ok(response) => {
					let is_healthy = response.into_inner().healthy;

					if !is_healthy {
						warn!("[MountainEnvironment] Air health check returned unhealthy");
					}

					is_healthy
				},
				Err(error) => {
					warn!("[MountainEnvironment] Air health check failed: {}", error);
					false
				},
			}
		} else {
			info!("[MountainEnvironment] No Air client configured");
			false
		}
	}

	/// Returns whether Air is available and ready.
	#[cfg(not(feature = "AirIntegration"))]
	pub async fn IsAirAvailable(&self) -> bool {
		false
	}
}

impl Environment for MountainEnvironment {}

#[async_trait]
impl ExtensionManagementService for MountainEnvironment {
	async fn ScanForExtensions(&self) -> Result<(), CommonError> {
		warn!("[ExtensionManagementService] ScanForExtensions is a stub.");

		Err(CommonError::NotImplemented { FeatureName:"ScanForExtensions".into() })
	}

	async fn GetExtensions(&self) -> Result<Vec<Value>, CommonError> {
		let ScannedExtensionsGuard = self
			.ApplicationState
			.ScannedExtensions
			.lock()
			.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;

		let Extensions:Vec<Value> = ScannedExtensionsGuard
			.values()
			.map(|ext| serde_json::to_value(ext).unwrap_or(Value::Null))
			.collect();

		Ok(Extensions)
	}

	async fn GetExtension(&self, _id:String) -> Result<Option<Value>, CommonError> {
		warn!("[ExtensionManagementService] GetExtension is a stub.");

		Err(CommonError::NotImplemented { FeatureName:"GetExtension".into() })
	}
}

// --- Capability Requirement Implementations (DI) ---

impl Requires<dyn CommandExecutor> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn CommandExecutor> { Arc::new(self.clone()) }
}

impl Requires<dyn ConfigurationProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ConfigurationProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn ConfigurationInspector> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ConfigurationInspector> { Arc::new(self.clone()) }
}

impl Requires<dyn CustomEditorProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn CustomEditorProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn DiagnosticManager> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn DiagnosticManager> { Arc::new(self.clone()) }
}

impl Requires<dyn DocumentProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn DocumentProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn FileSystemReader> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemReader> { Arc::new(self.clone()) }
}

impl Requires<dyn FileSystemWriter> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemWriter> { Arc::new(self.clone()) }
}

impl Requires<dyn IPCProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn IPCProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn LanguageFeatureProviderRegistry> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn LanguageFeatureProviderRegistry> { Arc::new(self.clone()) }
}

impl Requires<dyn OutputChannelManager> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn OutputChannelManager> { Arc::new(self.clone()) }
}

impl Requires<dyn SourceControlManagementProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SourceControlManagementProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn SecretProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SecretProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn StatusBarProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn StatusBarProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn StorageProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn StorageProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn SynchronizationProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SynchronizationProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn TerminalProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TerminalProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn TestController> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TestController> { Arc::new(self.clone()) }
}

impl Requires<dyn TreeViewProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TreeViewProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn UserInterfaceProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn UserInterfaceProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn WebViewProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WebViewProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn WorkSpaceProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WorkSpaceProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn WorkSpaceEditApplier> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WorkSpaceEditApplier> { Arc::new(self.clone()) }
}

impl Requires<dyn ExtensionManagementService> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ExtensionManagementService> { Arc::new(self.clone()) }
}

impl Requires<dyn DebugService> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn DebugService> { Arc::new(self.clone()) }
}

impl Requires<dyn KeybindingProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn KeybindingProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn SearchProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SearchProvider> { Arc::new(self.clone()) }
}
