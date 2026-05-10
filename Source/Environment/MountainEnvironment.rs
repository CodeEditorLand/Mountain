use std::sync::Arc;

// Import Air service client when Air integration is enabled
#[cfg(feature = "AirIntegration")]
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
		State::ApplicationState::ApplicationState,
	},
	dev_log,
};
// Import the macro for generating trait implementations
// Note: Macros annotated with #[macro_export] are available at crate root
use crate::impl_provider;

/// The concrete `Environment` for the Mountain application.
#[derive(Clone)]
pub struct MountainEnvironment {
	pub ApplicationHandle:AppHandle<Wry>,

	pub ApplicationState:Arc<ApplicationState>,

	/// Optional Air client for cloud-based services.
	/// When provided, providers like SecretProvider and UpdateService can
	/// delegate to Air.
	#[cfg(feature = "AirIntegration")]
	pub AirClient:Option<AirServiceClient<tonic::transport::Channel>>,
}

impl MountainEnvironment {
	/// Creates a new `MountainEnvironment` instance.
	#[allow(unused_mut)]
	pub fn Create(ApplicationHandle:AppHandle<Wry>, ApplicationState:Arc<ApplicationState>) -> Self {
		dev_log!("lifecycle", "[MountainEnvironment] New instance created.");

		#[cfg(feature = "AirIntegration")]
		{
			Self { ApplicationHandle, ApplicationState, AirClient:None }
		}

		#[cfg(not(feature = "AirIntegration"))]
		{
			Self { ApplicationHandle, ApplicationState }
		}
	}

	/// Creates a new `MountainEnvironment` instance with an optional Air
	/// client. When AirClient is provided, providers can delegate to Air for
	/// cloud-based services.
	#[cfg(feature = "AirIntegration")]
	pub fn CreateWithAir(
		ApplicationHandle:AppHandle<Wry>,

		ApplicationState:Arc<ApplicationState>,

		AirClient:Option<AirServiceClient<tonic::transport::Channel>>,
	) -> Self {
		dev_log!(
			"lifecycle",
			"[MountainEnvironment] New instance created with Air client: {}",
			AirClient.is_some()
		);

		Self { ApplicationHandle, ApplicationState, AirClient }
	}

	/// Updates the Air client for this environment.
	/// This allows dynamically switching between Air and local services.
	#[cfg(feature = "AirIntegration")]
	pub fn SetAirClient(&mut self, AirClient:Option<AirServiceClient<tonic::transport::Channel>>) {
		dev_log!("lifecycle", "[MountainEnvironment] Air client updated: {}", AirClient.is_some());

		self.AirClient = AirClient;
	}

	/// Returns whether Air is available and ready.
	#[cfg(feature = "AirIntegration")]
	pub async fn IsAirAvailable(&self) -> bool {
		// TODO: Implement proper health check when AirClient wrapper is available
		// The raw gRPC client requires &mut self for health_check, but
		// MountainEnvironment stores an immutable reference. This will be fixed when
		// the AirClient wrapper is properly integrated.
		if let Some(_AirClient) = &self.AirClient {
			// For now, assume Air is available if the client exists
			dev_log!(
				"lifecycle",
				"[MountainEnvironment] Air client configured (health check disabled pending integration)"
			);

			true
		} else {
			dev_log!("lifecycle", "[MountainEnvironment] No Air client configured");

			false
		}
	}

	/// Returns whether Air is available and ready.
	#[cfg(not(feature = "AirIntegration"))]
	pub async fn IsAirAvailable(&self) -> bool { false }

	/// Scans a directory for extensions and returns their package.json data
	async fn ScanExtensionDirectory(&self, path:&std::path::PathBuf) -> Result<Vec<serde_json::Value>, CommonError> {
		use std::fs;

		let mut extensions = Vec::new();

		// Check if directory exists
		if !path.exists() || !path.is_dir() {
			dev_log!(
				"lifecycle",
				"warn: [ExtensionManagementService] Extension directory does not exist: {:?}",
				path
			);

			return Ok(extensions);
		}

		// Read directory contents
		let entries = fs::read_dir(path).map_err(|error| {
			CommonError::FileSystemIO {
				Path:path.clone(),
				Description:format!("Failed to read extension directory: {}", error),
			}
		})?;

		for entry in entries {
			let entry = entry.map_err(|error| {
				CommonError::FileSystemIO {
					Path:path.clone(),
					Description:format!("Failed to read directory entry: {}", error),
				}
			})?;

			let entry_path = entry.path();

			if entry_path.is_dir() {
				// Look for package.json in the extension directory
				let package_json_path = entry_path.join("package.json");

				if package_json_path.exists() {
					match fs::read_to_string(&package_json_path) {
						Ok(content) => {
							match serde_json::from_str::<Value>(&content) {
								Ok(mut package_json) => {
									// Add extension location information
									if let Some(obj) = package_json.as_object_mut() {
										obj.insert(
											"ExtensionLocation".to_string(),
											Value::String(entry_path.to_string_lossy().to_string()),
										);
									}

									extensions.push(package_json);

									dev_log!(
										"lifecycle",
										"[ExtensionManagementService] Found extension at: {:?}",
										entry_path
									);
								},

								Err(error) => {
									dev_log!(
										"lifecycle",
										"warn: [ExtensionManagementService] Failed to parse package.json at {:?}: {}",
										package_json_path,
										error
									);
								},
							}
						},

						Err(error) => {
							dev_log!(
								"lifecycle",
								"warn: [ExtensionManagementService] Failed to read package.json at {:?}: {}",
								package_json_path,
								error
							);
						},
					}
				}
			}
		}

		Ok(extensions)
	}
}

impl Environment for MountainEnvironment {}

#[async_trait]
impl ExtensionManagementService for MountainEnvironment {
	async fn ScanForExtensions(&self) -> Result<(), CommonError> {
		dev_log!("lifecycle", "[ExtensionManagementService] Scanning for extensions...");

		// Get the extension scan paths from ApplicationState
		let ScanPaths:Vec<std::path::PathBuf> = {
			let ScanPathsGuard = self
				.ApplicationState
				.Extension
				.Registry
				.ExtensionScanPaths
				.lock()
				.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;

			ScanPathsGuard.clone()
		};

		let mut extensions = Vec::new();

		// Scan each extension directory
		for path in ScanPaths {
			if let Ok(mut scan_result) = self.ScanExtensionDirectory(&path).await {
				extensions.append(&mut scan_result);
			}
		}

		// Update ApplicationState with scanned extensions
		let mut ScannedExtensionsGuard = self
			.ApplicationState
			.Extension
			.ScannedExtensions
			.ScannedExtensions
			.lock()
			.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;

		ScannedExtensionsGuard.clear();

		for extension in extensions {
			// The scanner returns camelCase JSON (serde rename_all = "camelCase").
			// Deserialize directly into ExtensionDescriptionStateDTO.
			match serde_json::from_value::<ExtensionDescriptionStateDTO>(extension.clone()) {
				Ok(Dto) => {
					// Use identifier.value or fall back to name
					let Key = Dto
						.Identifier
						.as_object()
						.and_then(|O| O.get("value"))
						.and_then(|V| V.as_str())
						.unwrap_or(&Dto.Name)
						.to_string();

					if !Key.is_empty() {
						ScannedExtensionsGuard.insert(Key, Dto);
					}
				},

				Err(Error) => {
					let Name = extension.get("name").and_then(|V| V.as_str()).unwrap_or("?");

					dev_log!(
						"lifecycle",
						"warn: [ExtensionManagementService] Failed to parse extension '{}': {}",
						Name,
						Error
					);
				},
			}
		}

		dev_log!(
			"lifecycle",
			"[ExtensionManagementService] Found {} extensions",
			ScannedExtensionsGuard.len()
		);

		Ok(())
	}

	async fn GetExtensions(&self) -> Result<Vec<Value>, CommonError> {
		let ScannedExtensionsGuard = self
			.ApplicationState
			.Extension
			.ScannedExtensions
			.ScannedExtensions
			.lock()
			.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;

		let GuardLen = ScannedExtensionsGuard.len();

		let Extensions:Vec<Value> = ScannedExtensionsGuard
			.values()
			.map(|ext| serde_json::to_value(ext).unwrap_or(Value::Null))
			.collect();

		let SerializedCount = Extensions.iter().filter(|v| !v.is_null()).count();

		dev_log!(
			"lifecycle",
			"[MountainEnvironment] GetExtensions: ScannedExtensions map={} entries, serialized={} non-null",
			GuardLen,
			SerializedCount
		);

		Ok(Extensions)
	}

	async fn GetExtension(&self, id:String) -> Result<Option<Value>, CommonError> {
		let ScannedExtensionsGuard = self
			.ApplicationState
			.Extension
			.ScannedExtensions
			.ScannedExtensions
			.lock()
			.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;

		if let Some(extension_dto) = ScannedExtensionsGuard.get(&id) {
			// Convert ExtensionDescriptionStateDTO back to JSON Value
			let mut extension_value = serde_json::Map::new();

			extension_value.insert("Identifier".to_string(), extension_dto.Identifier.clone());

			extension_value.insert("Name".to_string(), Value::String(extension_dto.Name.clone()));

			extension_value.insert("Version".to_string(), Value::String(extension_dto.Version.clone()));

			extension_value.insert("Publisher".to_string(), Value::String(extension_dto.Publisher.clone()));

			extension_value.insert("Engines".to_string(), extension_dto.Engines.clone());

			if let Some(main) = &extension_dto.Main {
				extension_value.insert("Main".to_string(), Value::String(main.clone()));
			}

			if let Some(browser) = &extension_dto.Browser {
				extension_value.insert("Browser".to_string(), Value::String(browser.clone()));
			}

			if let Some(module_type) = &extension_dto.ModuleType {
				extension_value.insert("ModuleType".to_string(), Value::String(module_type.clone()));
			}

			extension_value.insert("IsBuiltin".to_string(), Value::Bool(extension_dto.IsBuiltin));

			extension_value.insert("IsUnderDevelopment".to_string(), Value::Bool(extension_dto.IsUnderDevelopment));

			extension_value.insert("ExtensionLocation".to_string(), extension_dto.ExtensionLocation.clone());

			if let Some(activation_events) = &extension_dto.ActivationEvents {
				let events:Vec<Value> = activation_events.iter().map(|e| Value::String(e.clone())).collect();

				extension_value.insert("ActivationEvents".to_string(), Value::Array(events));
			}

			if let Some(contributes) = &extension_dto.Contributes {
				extension_value.insert("Contributes".to_string(), contributes.clone());
			}

			Ok(Some(Value::Object(extension_value)))
		} else {
			Ok(None)
		}
	}
}

// Capability requirement implementations (DI) - all generated by impl_provider!

// Command and Configuration
impl_provider!(CommandExecutor);

impl_provider!(ConfigurationProvider);

impl_provider!(ConfigurationInspector);

// Custom Editor and Debug
impl_provider!(CustomEditorProvider);

impl_provider!(DebugService);

// Document and Diagnostic
impl_provider!(DocumentProvider);

impl_provider!(DiagnosticManager);

// File System
impl_provider!(FileSystemReader);

impl_provider!(FileSystemWriter);

impl_provider!(FileWatcherProvider);

// IPC and Keybinding
impl_provider!(IPCProvider);

impl_provider!(KeybindingProvider);

// Language Features and Output
impl_provider!(LanguageFeatureProviderRegistry);

impl_provider!(OutputChannelManager);

// Secret and SCM
impl_provider!(SecretProvider);

impl_provider!(SourceControlManagementProvider);

// Status Bar and Storage
impl_provider!(StatusBarProvider);

impl_provider!(StorageProvider);

// Synchronization and Terminal
impl_provider!(SynchronizationProvider);

impl_provider!(TerminalProvider);

// Test and Tree View
impl_provider!(TestController);

impl_provider!(TreeViewProvider);

// UI and Webview
impl_provider!(UserInterfaceProvider);

impl_provider!(WebviewProvider);

// Workspace
impl_provider!(WorkspaceProvider);

impl_provider!(WorkspaceEditApplier);

// Extension Management and Search
impl_provider!(ExtensionManagementService);

impl_provider!(SearchProvider);
