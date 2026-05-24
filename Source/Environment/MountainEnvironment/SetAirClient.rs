//! `MountainEnvironment::SetAirClient`

use super::Struct;
use std::sync::Arc;
use AirLibrary::Vine::Generated::air::air_service_client::AirServiceClient;
use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	Configuration::{ConfigurationInspector::ConfigurationInspector, ConfigurationProvider::ConfigurationProvider},
	CustomEditor::CustomEditorProvider::CustomEditorProvider,
	Debug::DebugService::DebugService,
	Diagnostic::DiagnosticManager::DiagnosticManager,
	Document::DocumentProvider::DocumentProvider,
	Environment::{Environment::Environment, Requires::Requires},
	Error::CommonError::CommonError,
	ExtensionManagement::ExtensionManagementService::ExtensionManagementService,
	FileSystem::{
		FileSystemReader::FileSystemReader,
		FileSystemWriter::FileSystemWriter,
		FileWatcherProvider::FileWatcherProvider,
	},
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
	Webview::WebviewProvider::WebviewProvider,
	Workspace::{WorkspaceEditApplier::WorkspaceEditApplier, WorkspaceProvider::WorkspaceProvider},
};
use async_trait::async_trait;
use serde_json::Value;
use tauri::{AppHandle, Wry};
use crate::{
	ApplicationState::{
		DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
		Struct::ApplicationState::ApplicationState,
	},
	dev_log,
};
use crate::impl_provider;

pub fn Fn(This:&mut Struct, AirClient:Option<AirServiceClient<tonic::transport::Channel>>) {
		dev_log!("lifecycle", "[MountainEnvironment] Air client updated: {}", AirClient.is_some());

		This.AirClient = AirClient;
	}
