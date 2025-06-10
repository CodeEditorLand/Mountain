//! Defines the concrete `MountainEnvironment` struct, the central context and
//! dependency injection container for the `Mountain` application.

use std::sync::Arc;

use Common::{
	command::CommandExecutor,
	configuration::ConfigurationProvider,
	custom_editor::CustomEditorProvider,
	diagnostic::DiagnosticsProvider,
	document::DocumentsProvider,
	environment::{Environment, Requires},
	fs::{FsReader, FsWriter},
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
use tauri::{AppHandle, Wry};

use crate::environment::{
	CommandProvider::CommandEnvironment,
	ConfigurationProvider::ConfigurationEnvironment,
	CustomEditorProvider::CustomEditorEnvironment,
	DiagnosticProvider::DiagnosticEnvironment,
	DocumentProvider::DocumentEnvironment,
	FsProvider::FsEnvironment,
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

/// The concrete environment for the Mountain application.
///
/// This struct acts as a top-level container for all domain-specific
/// sub-environments. It implements the `Requires<T>` trait for every service
/// capability by delegating the request to the appropriate sub-environment.
/// This pattern promotes separation of concerns, keeping the implementation
/// details of each service domain isolated.
#[derive(Clone)]
pub struct MountainEnvironment {
	pub CommandEnvironment:Arc<CommandEnvironment>,
	pub ConfigurationEnvironment:Arc<ConfigurationEnvironment>,
	pub CustomEditorEnvironment:Arc<CustomEditorEnvironment>,
	pub DiagnosticEnvironment:Arc<DiagnosticEnvironment>,
	pub DocumentEnvironment:Arc<DocumentEnvironment>,
	pub FsEnvironment:Arc<FsEnvironment>,
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
	/// Creates a new `MountainEnvironment` by instantiating all of its
	/// sub-environments.
	pub fn New(AppHandle:AppHandle<Wry>) -> Self {
		info!("[MountainEnvironment] New instance created.");
		Self {
			CommandEnvironment:Arc::new(CommandEnvironment::New(AppHandle.clone())),
			ConfigurationEnvironment:Arc::new(ConfigurationEnvironment::New(AppHandle.clone())),
			CustomEditorEnvironment:Arc::new(CustomEditorEnvironment::New(AppHandle.clone())),
			DiagnosticEnvironment:Arc::new(DiagnosticEnvironment::New(AppHandle.clone())),
			DocumentEnvironment:Arc::new(DocumentEnvironment::New(AppHandle.clone())),
			FsEnvironment:Arc::new(FsEnvironment::New(AppHandle.clone())),
			IpcEnvironment:Arc::new(IpcEnvironment::New(AppHandle.clone())),
			LanguageFeatureEnvironment:Arc::new(LanguageFeatureEnvironment::New(AppHandle.clone())),
			OutputEnvironment:Arc::new(OutputEnvironment::New(AppHandle.clone())),
			ScmEnvironment:Arc::new(ScmEnvironment::New(AppHandle.clone())),
			SecretEnvironment:Arc::new(SecretEnvironment::New(AppHandle.clone())),
			StatusBarEnvironment:Arc::new(StatusBarEnvironment::New(AppHandle.clone())),
			StorageEnvironment:Arc::new(StorageEnvironment::New(AppHandle.clone())),
			SyncEnvironment:Arc::new(SyncEnvironment::New(AppHandle.clone())),
			TerminalEnvironment:Arc::new(TerminalEnvironment::New(AppHandle.clone())),
			TestEnvironment:Arc::new(TestEnvironment::New(AppHandle.clone())),
			TreeViewEnvironment:Arc::new(TreeViewEnvironment::New(AppHandle.clone())),
			UiEnvironment:Arc::new(UiEnvironment::New(AppHandle.clone())),
			WebviewEnvironment:Arc::new(WebviewEnvironment::New(AppHandle.clone())),
			WorkspaceEnvironment:Arc::new(WorkspaceEnvironment::New(AppHandle.clone())),
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
impl Requires<Arc<dyn FsReader>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FsReader> { self.FsEnvironment.clone() }
}
impl Requires<Arc<dyn FsWriter>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn FsWriter> { self.FsEnvironment.clone() }
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
