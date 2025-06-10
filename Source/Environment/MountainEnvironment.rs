// Defines the concrete `MountainEnvironment` struct, the central context and
// dependency injection container for the `Mountain` application.

use std::sync::Arc;

use Common::{
	command::CommandExecutor,
	configuration::ConfigurationProvider,
	custom_editor::CustomEditorProvider,
	diagnostic::DiagnosticsProvider,
	document::DocumentsProvider,
	environment::{Environment, Requires},
	fs::{FileSystemReader, FileSystemWriter},
	ipc::IpcProvider,
	language_feature::LanguageFeatureProviderRegistry,
	output::OutputProvider,
	scm::ScmProvider,
	secret::SecretsProvider,
	status_bar::StatusBarProvider,
	storage::StorageProvider,
	sync::SyncProvider,
	terminal::TerminalProvider,
	test::TestingProvider,
	tree_view::TreeViewProvider,
	ui::UiProvider,
	webview::WebviewProvider,
	workspace::WorkspaceProvider,
};
use log::info;
use tauri::{ApplicationHandle, Wry};

use crate::environment::{
	CommandProvider::CommandEnvironment,
	ConfigurationProvider::ConfigurationEnvironment,
	CustomEditorProvider::CustomEditorEnvironment,
	DiagnosticProvider::DiagnosticEnvironment,
	DocumentProvider::DocumentEnvironment,
	FileSystemProvider::FileSystemEnvironment,
	IpcProvider::IpcEnvironment,
	LanguageFeatureProvider::LanguageFeatureEnvironment,
	OutputProvider::OutputEnvironment,
	ScmProvider::ScmEnvironment,
	SecretProvider::SecretEnvironment,
	StatusBarProvider::StatusBarEnvironment,
	StorageProvider::StorageEnvironment,
	SyncProvider::SyncEnvironment,
	TerminalProvider::TerminalEnvironment,
	TestProvider::TestEnvironment,
	TreeViewProvider::TreeViewEnvironment,
	UiProvider::UiEnvironment,
	WebviewProvider::WebviewEnvironment,
	WorkspaceProvider::WorkspaceEnvironment,
};

// The concrete environment for the Mountain application.
//
// This struct acts as a top-level container for all domain-specific
// sub-environments. It implements the `Requires<T>` trait for every service
// capability by delegating the request to the appropriate sub-environment.
// This pattern promotes separation of concerns, keeping the implementation
// details of each service domain isolated.
#[derive(Clone)]
pub struct MountainEnvironment {
	pub CommandEnvironment:Arc<CommandEnvironment>,
	pub ConfigurationEnvironment:Arc<ConfigurationEnvironment>,
	pub CustomEditorEnvironment:Arc<CustomEditorEnvironment>,
	pub DiagnosticEnvironment:Arc<DiagnosticEnvironment>,
	pub DocumentEnvironment:Arc<DocumentEnvironment>,
	pub FileSystemEnvironment:Arc<FileSystemEnvironment>,
	pub IpcEnvironment:Arc<IpcEnvironment>,
	pub LanguageFeatureEnvironment:Arc<LanguageFeatureEnvironment>,
	pub OutputEnvironment:Arc<OutputEnvironment>,
	pub ScmEnvironment:Arc<ScmEnvironment>,
	pub SecretEnvironment:Arc<SecretEnvironment>,
	pub StatusBarEnvironment:Arc<StatusBarEnvironment>,
	pub StorageEnvironment:Arc<StorageEnvironment>,
	pub SyncEnvironment:Arc<SyncEnvironment>,
	pub TerminalEnvironment:Arc<TerminalEnvironment>,
	pub TestEnvironment:Arc<TestEnvironment>,
	pub TreeViewEnvironment:Arc<TreeViewEnvironment>,
	pub UiEnvironment:Arc<UiEnvironment>,
	pub WebviewEnvironment:Arc<WebviewEnvironment>,
	pub WorkspaceEnvironment:Arc<WorkspaceEnvironment>,
}

impl MountainEnvironment {
	// Creates a new `MountainEnvironment` by instantiating all of its
	// sub-environments.
	pub fn New(ApplicationHandle:ApplicationHandle<Wry>) -> Self {
		info!("[MountainEnvironment] New instance created.");
		Self {
			CommandEnvironment:Arc::new(CommandEnvironment::New(ApplicationHandle.clone())),
			ConfigurationEnvironment:Arc::new(ConfigurationEnvironment::New(ApplicationHandle.clone())),
			CustomEditorEnvironment:Arc::new(CustomEditorEnvironment::New(ApplicationHandle.clone())),
			DiagnosticEnvironment:Arc::new(DiagnosticEnvironment::New(ApplicationHandle.clone())),
			DocumentEnvironment:Arc::new(DocumentEnvironment::New(ApplicationHandle.clone())),
			FileSystemEnvironment:Arc::new(FileSystemEnvironment::New(ApplicationHandle.clone())),
			IpcEnvironment:Arc::new(IpcEnvironment::New(ApplicationHandle.clone())),
			LanguageFeatureEnvironment:Arc::new(LanguageFeatureEnvironment::New(ApplicationHandle.clone())),
			OutputEnvironment:Arc::new(OutputEnvironment::New(ApplicationHandle.clone())),
			ScmEnvironment:Arc::new(ScmEnvironment::New(ApplicationHandle.clone())),
			SecretEnvironment:Arc::new(SecretEnvironment::New(ApplicationHandle.clone())),
			StatusBarEnvironment:Arc::new(StatusBarEnvironment::New(ApplicationHandle.clone())),
			StorageEnvironment:Arc::new(StorageEnvironment::New(ApplicationHandle.clone())),
			SyncEnvironment:Arc::new(SyncEnvironment::New(ApplicationHandle.clone())),
			TerminalEnvironment:Arc::new(TerminalEnvironment::New(ApplicationHandle.clone())),
			TestEnvironment:Arc::new(TestEnvironment::New(ApplicationHandle.clone())),
			TreeViewEnvironment:Arc::new(TreeViewEnvironment::New(ApplicationHandle.clone())),
			UiEnvironment:Arc::new(UiEnvironment::New(ApplicationHandle.clone())),
			WebviewEnvironment:Arc::new(WebviewEnvironment::New(ApplicationHandle.clone())),
			WorkspaceEnvironment:Arc::new(WorkspaceEnvironment::New(ApplicationHandle.clone())),
		}
	}
}

impl Environment for MountainEnvironment {}

// --- Capability Requirement Implementations (Delegation) --- //

impl Requires<Arc<dyn CommandExecutor>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn CommandExecutor> { self.CommandEnvironment.clone() }
}
impl Requires<Arc<dyn ConfigurationProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ConfigurationProvider> { self.ConfigurationEnvironment.clone() }
}
impl Requires<Arc<dyn CustomEditorProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn CustomEditorProvider> { self.CustomEditorEnvironment.clone() }
}
impl Requires<Arc<dyn DiagnosticsProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn DiagnosticsProvider> { self.DiagnosticEnvironment.clone() }
}
impl Requires<Arc<dyn DocumentsProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn DocumentsProvider> { self.DocumentEnvironment.clone() }
}
impl Requires<Arc<dyn FileSystemReader>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemReader> { self.FileSystemEnvironment.clone() }
}
impl Requires<Arc<dyn FileSystemWriter>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FileSystemWriter> { self.FileSystemEnvironment.clone() }
}
impl Requires<Arc<dyn IpcProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn IpcProvider> { self.IpcEnvironment.clone() }
}
impl Requires<Arc<dyn LanguageFeatureProviderRegistry>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn LanguageFeatureProviderRegistry> { self.LanguageFeatureEnvironment.clone() }
}
impl Requires<Arc<dyn OutputProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn OutputProvider> { self.OutputEnvironment.clone() }
}
impl Requires<Arc<dyn ScmProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ScmProvider> { self.ScmEnvironment.clone() }
}
impl Requires<Arc<dyn SecretsProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SecretsProvider> { self.SecretEnvironment.clone() }
}
impl Requires<Arc<dyn StatusBarProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn StatusBarProvider> { self.StatusBarEnvironment.clone() }
}
impl Requires<Arc<dyn StorageProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn StorageProvider> { self.StorageEnvironment.clone() }
}
impl Requires<Arc<dyn SyncProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SyncProvider> { self.SyncEnvironment.clone() }
}
impl Requires<Arc<dyn TerminalProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TerminalProvider> { self.TerminalEnvironment.clone() }
}
impl Requires<Arc<dyn TestingProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TestingProvider> { self.TestEnvironment.clone() }
}
impl Requires<Arc<dyn TreeViewProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TreeViewProvider> { self.TreeViewEnvironment.clone() }
}
impl Requires<Arc<dyn UiProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn UiProvider> { self.UiEnvironment.clone() }
}
impl Requires<Arc<dyn WebviewProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WebviewProvider> { self.WebviewEnvironment.clone() }
}
impl Requires<Arc<dyn WorkspaceProvider>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WorkspaceProvider> { self.WorkspaceEnvironment.clone() }
}
