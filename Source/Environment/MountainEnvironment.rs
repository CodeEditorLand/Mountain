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
	Diagnostic::DiagnosticManager::DiagnosticManager,
	Document::DocumentProvider::DocumentProvider,
	Environment::{Environment::Environment, Requires::Requires},
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
	IPC::IPCProvider::IPCProvider,
	LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	Output::OutputChannelManager::OutputChannelManager,
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
use log::info;
use tauri::{AppHandle, Manager, Wry};

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
		let ApplicationState = Arc::new(ApplicationHandle.state::<ApplicationState>().inner().clone());
		Self { ApplicationHandle, ApplicationState }
	}
}

impl Environment for MountainEnvironment {}

// --- Capability Requirement Implementations (DI) ---
// This is the core of the DI system. The `MountainEnvironment` itself
// implements every provider trait, so when an effect requires a capability, we
// provide a clone of the environment, which satisfies the trait bound.

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
