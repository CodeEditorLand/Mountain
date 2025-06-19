//! # MountainEnvironment
//!
//! Defines the concrete `MountainEnvironment` struct, which serves as the
//! central context and dependency injection container for the `Mountain`
//! application.

use std::sync::Arc;

use Common::{
	Command::CommandExecutor,
	Configuration::{ConfigurationInspector, ConfigurationProvider},
	CustomEditor::CustomEditorProvider,
	Diagnostic::DiagnosticManager,
	Document::DocumentProvider,
	Environment::{Environment, Requires},
	FileSystem::{FileSystemReader, FileSystemWriter},
	IPC::IPCProvider,
	LanguageFeature::LanguageFeatureProviderRegistry,
	Output::OutputChannelManager,
	Secret::SecretProvider,
	SourceControlManagement::SourceControlManagementProvider,
	StatusBar::StatusBarProvider,
	Storage::StorageProvider,
	Synchronization::SynchronizationProvider,
	Terminal::TerminalProvider,
	Testing::TestController,
	TreeView::TreeViewProvider,
	UserInterface::UserInterfaceProvider,
	WebView::WebViewProvider,
	Workspace::{WorkspaceEditApplier, WorkspaceProvider},
};
use log::info;
use tauri::{AppHandle, Manager, Runtime};

use crate::ApplicationState::ApplicationState::ApplicationState;

/// The concrete `Environment` for the Mountain application.
///
/// This struct acts as the top-level dependency injection container. It holds a
/// handle to the core `ApplicationState` and the Tauri `AppHandle`. It
/// implements all `Requires<Trait>` from the `Common` crate by simply cloning
/// itself, as it also implements all the provider traits directly.
#[derive(Clone)]
pub struct MountainEnvironment {
	pub ApplicationHandle:AppHandle,
	pub ApplicationState:Arc<ApplicationState>,
}

impl MountainEnvironment {
	/// Creates a new `MountainEnvironment` instance.
	pub fn Create(ApplicationHandle:AppHandle) -> Self {
		info!("[MountainEnvironment] New instance created.");
		let ApplicationState = Arc::new(ApplicationHandle.state::<ApplicationState>().inner().clone());
		Self { ApplicationHandle, ApplicationState }
	}
}

impl Environment for MountainEnvironment {}

// --- Capability Requirement Implementations (DI) --- //
// This is the core of the DI system. Any effect needing a capability will
// receive an `Arc<MountainEnvironment>`, which satisfies the trait bounds
// because `MountainEnvironment` implements all the provider traits below.

impl Requires<Arc<dyn CommandExecutor + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn CommandExecutor + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn ConfigurationProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ConfigurationProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn ConfigurationInspector + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ConfigurationInspector + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn CustomEditorProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn CustomEditorProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn DiagnosticManager + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn DiagnosticManager + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn DocumentProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn DocumentProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn FileSystemReader + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemReader + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn FileSystemWriter + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemWriter + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn IPCProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn IPCProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn LanguageFeatureProviderRegistry + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn LanguageFeatureProviderRegistry + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn OutputChannelManager + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn OutputChannelManager + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn SourceControlManagementProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SourceControlManagementProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn SecretProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SecretProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn StatusBarProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn StatusBarProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn StorageProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn StorageProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn SynchronizationProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SynchronizationProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn TerminalProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TerminalProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn TestController + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TestController + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn TreeViewProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TreeViewProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn UserInterfaceProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn UserInterfaceProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn WebViewProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WebViewProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn WorkspaceProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WorkspaceProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn WorkspaceEditApplier + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WorkspaceEditApplier + Send + Sync> { Arc::new(self.clone()) }
}
