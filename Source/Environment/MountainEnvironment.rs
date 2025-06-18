// @module MountainEnvironment
// @description Defines the concrete `MountainEnvironment` struct, the central
// context and dependency injection container for the `Mountain` application.

use std::sync::Arc;

use Common::{
	command::CommandExecutor,
	config::{ConfigInspector, ConfigProvider},
	custom_editor::CustomEditorProvider,
	diagnostic::DiagnosticsProvider,
	document::DocumentsProvider,
	Environment::{Environment, Requires},
	fs::{FileSystemReader, FileSystemWriter},
	IPC::IpcProvider,
	language_feature::LanguageFeatureProviderRegistry,
	output::OutputProvider,
	scm::ScmProvider,
	secret::SecretsProvider,
	status_bar::StatusBarProvider,
	storage::StorageProvider,
	sync::SyncProvider,
	terminal::TerminalProvider,
	test::TestProvider,
	tree_view::TreeViewProvider,
	ui::UiProvider,
	webview::WebviewProvider,
	workspace::{WorkspaceEditApplier, WorkspaceProvider},
};
use log::info;
use tauri::{AppHandle, Wry};

/// The concrete Environment for the Mountain application.
///
/// This struct acts as the top-level dependency injection container. It
/// implements all `Requires<Trait>` from the `Common` crate by simply cloning
/// itself, as it also implements all the provider traits directly by delegating
/// to the appropriate `Handler` modules.
#[derive(Clone)]
pub struct MountainEnvironment {
	pub ApplicationHandle:AppHandle<Wry>,
}

impl MountainEnvironment {
	/// Creates a new `MountainEnvironment` instance.
	pub fn New(app_handle:AppHandle<Wry>) -> Self {
		info!("[MountainEnvironment] New instance created.");
		Self { ApplicationHandle:app_handle }
	}
}

impl Environment for MountainEnvironment {}

// --- Capability Requirement Implementations (Delegation) --- //
// This is the core of the DI system. Any effect needing a capability will
// receive an `Arc<MountainEnvironment>`, which satisfies the trait bounds.

impl Requires<Arc<dyn CommandExecutor + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn CommandExecutor + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn ConfigProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ConfigProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn ConfigInspector + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ConfigInspector + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn CustomEditorProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn CustomEditorProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn DiagnosticsProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn DiagnosticsProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn DocumentsProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn DocumentsProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn FileSystemReader + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemReader + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn FileSystemWriter + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemWriter + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn IpcProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn IpcProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn LanguageFeatureProviderRegistry + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn LanguageFeatureProviderRegistry + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn OutputProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn OutputProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn ScmProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ScmProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn SecretsProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SecretsProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn StatusBarProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn StatusBarProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn StorageProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn StorageProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn SyncProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SyncProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn TerminalProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TerminalProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn TestProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TestProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn TreeViewProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TreeViewProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn UiProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn UiProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn WebviewProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WebviewProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn WorkspaceProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WorkspaceProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn WorkspaceEditApplier + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WorkspaceEditApplier + Send + Sync> { Arc::new(self.clone()) }
}
