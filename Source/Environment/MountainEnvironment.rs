// File: Mountain/Source/Environment/MountainEnvironment.rs

//! # MountainEnvironment
//!
//! Defines the concrete `MountainEnvironment` struct, which serves as the
//! central context and dependency injection container for the `Mountain`
//! application.

use std::sync::Arc;

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
use log::info;
use serde_json::Value;
use tauri::{AppHandle, Manager, Wry};
use url::Url;

use crate::ApplicationState::ApplicationState::ApplicationState;

/// The concrete `Environment` for the Mountain application.
#[derive(Clone)]
pub struct MountainEnvironment {
	pub ApplicationHandle:AppHandle<Wry>,
	pub ApplicationState:Arc<ApplicationState>,
}

impl MountainEnvironment {
	/// Creates a new `MountainEnvironment` instance.
	pub fn Create(ApplicationHandle:AppHandle<Wry>) -> Self {
		info!("[MountainEnvironment] New instance created.");
		let ApplicationState = ApplicationHandle.state::<Arc<ApplicationState>>().inner().clone();
		Self { ApplicationHandle, ApplicationState }
	}
}

impl Environment for MountainEnvironment {}

#[async_trait]
impl ExtensionManagementService for MountainEnvironment {
	async fn ScanForExtensions(&self) -> Result<(), CommonError> { todo!() }

	async fn GetExtensions(&self) -> Result<Vec<Value>, CommonError> { todo!() }

	async fn GetExtension(&self, _id:String) -> Result<Option<Value>, CommonError> { todo!() }
}
#[async_trait]
impl DebugService for MountainEnvironment {
	async fn RegisterDebugConfigurationProvider(
		&self,
		_debug_type:String,
		_provider_handle:u32,
		_extension_id:String,
	) -> Result<(), CommonError> {
		todo!()
	}

	async fn RegisterDebugAdapterDescriptorFactory(
		&self,
		_debug_type:String,
		_factory_handle:u32,
		_extension_id:String,
	) -> Result<(), CommonError> {
		todo!()
	}

	async fn StartDebugging(&self, _folder:Option<Url>, _configuration:Value) -> Result<String, CommonError> { todo!() }

	async fn SendCommand(&self, _session_id:String, _command:String, _args:Value) -> Result<Value, CommonError> {
		todo!()
	}
}
#[async_trait]
impl SearchProvider for MountainEnvironment {
	async fn TextSearch(&self, _query:Value, _options:Value) -> Result<Value, CommonError> { todo!() }
}

// --- Capability Requirement Implementations (DI) ---
// ... (The rest of the file remains the same)

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
