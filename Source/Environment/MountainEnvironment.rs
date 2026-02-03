// File: Mountain/Source/Environment/MountainEnvironment.rs
//
// # Architectural Role: Central DI Container and Application Context
//
// MountainEnvironment is the primary dependency injection (DI) container for
// the Mountain application. It implements all provider traits defined in the
// Common crate, acting as the central orchestrator that provides access to all
// platform services.
//
// # Responsibilities
//
// 1. **Dependency Injection Container**: Implements Requires<T> for all 19+
//    provider traits, enabling other components to request dependencies through
//    the Require() method.
//
// 2. **Application Lifecycle Management**: Holds references to Tauri AppHandle
//    and ApplicationState, managing the core application context and state.
//
// 3. **Air Integration**: Optionally manages the Air gRPC client for
//    cloud-based services when the AirIntegration feature is enabled. Enables
//    dynamic switching between local and cloud services.
//
// 4. **Extension Management**: Implements ExtensionManagementService for
//    discovering, scanning, and managing extensions in the system.
//
// 5. **Service Orchestration**: Acts as the central coordinator between all
//    providers (FileSystem, Document, Command, Configuration, IPC, etc.),
//    ensuring proper initialization and interaction.
//
// # Initialization Sequence
//
// 1. Create MountainEnvironment instance via Create() or CreateWithAir()
// 2. Provider instances are created lazily through Requires<T> traits
// 3. Each provider can access ApplicationState and AppHandle through self
// 4. Inter-provider communication is handled via IPCProvider or direct Rust
//    calls
//
// # Dependency Wiring
//
// All providers implement their respective traits from the Common crate.
// MountainEnvironment implements Requires<T> for each trait, returning an
// Arc-wrapped clone of itself. This enables circular dependencies and lazy
// initialization while maintaining type safety.
//
// # Patterns Borrowed from VSCode
//
// - **ServiceCollection Pattern**: Like VSCode's ServiceCollection,
//   MountainEnvironment registers and provides all services in a centralized
//   location.
//
// - **Lifecycle Management**: Similar to VSCode's IDisposable pattern,
//   resources are automatically managed through Arc reference counting.
//
// - **Extension Points**: Extension management follows VSCode's activation
//   event pattern, enabling lazy loading of extension services.
//
// # TODOs
//
// - [ ] Add telemetry integration for performance monitoring
// - [ ] Implement proper provider health checking
// - [ ] Add provider dependency validation on initialization
// - [ ] Consider async initialization for providers
// - [ ] Add circuit breaker pattern for external service calls (Air)
// - [ ] Implement graceful degradation when providers fail
// - [ ] Add metrics collection for provider usage
// - [ ] Consider provider initialization order dependencies

use std::sync::Arc;

// Import Air service client when Air integration is enabled
#[cfg(feature = "AirIntegration")]
use AirLibrary::Vine::Generated::Air::air_service_client::AirServiceClient;
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
	Webview::WebviewProvider::WebviewProvider,
	Workspace::{WorkspaceEditApplier::WorkspaceEditApplier, WorkspaceProvider::WorkspaceProvider},
};
use async_trait::async_trait;
use log::{info, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Wry};

use crate::ApplicationState::{
	ApplicationState::ApplicationState,
	DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
};

/// The concrete `Environment` for the Mountain application.
#[derive(Clone)]
pub struct MountainEnvironment {
	pub ApplicationHandle:AppHandle<Wry>,

	pub ApplicationState:Arc<ApplicationState>,

	/// Optional Air client for cloud-based services.
	/// When provided, providers like SecretProvider and UpdateService can
	/// delegate to Air.
	#[cfg(feature = "AirIntegration")]
	pub AirClient:Option<Arc<AirServiceClient<tonic::transport::Channel>>>,
}

impl MountainEnvironment {
	/// Creates a new `MountainEnvironment` instance.
	#[allow(unused_mut)]
	pub fn Create(ApplicationHandle:AppHandle<Wry>) -> Self {
		info!("[MountainEnvironment] New instance created.");

		let ApplicationState = ApplicationHandle.state::<Arc<ApplicationState>>().inner().clone();

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
		AirClient:Option<Arc<AirServiceClient<tonic::transport::Channel>>>,
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
	pub fn SetAirClient(&mut self, AirClient:Option<Arc<AirServiceClient<tonic::transport::Channel>>>) {
		info!("[MountainEnvironment] Air client updated: {}", AirClient.is_some());

		self.AirClient = AirClient;
	}

	/// Returns whether Air is available and ready.
	#[cfg(feature = "AirIntegration")]
	pub async fn IsAirAvailable(&self) -> bool {
		if let Some(AirClient) = &self.AirClient {
			use tonic::Request;
			use AirLibrary::Vine::Generated::Air::HealthCheckRequest;

			match AirClient.health_check(Request::new(HealthCheckRequest {})).await {
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
	pub async fn IsAirAvailable(&self) -> bool { false }

	/// Scans a directory for extensions and returns their package.json data
	async fn ScanExtensionDirectory(&self, path:&std::path::PathBuf) -> Result<Vec<serde_json::Value>, CommonError> {
		use std::fs;

		use serde_json::Value;

		let mut extensions = Vec::new();

		// Check if directory exists
		if !path.exists() || !path.is_dir() {
			warn!("[ExtensionManagementService] Extension directory does not exist: {:?}", path);
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
									info!("[ExtensionManagementService] Found extension at: {:?}", entry_path);
								},
								Err(error) => {
									warn!(
										"[ExtensionManagementService] Failed to parse package.json at {:?}: {}",
										package_json_path, error
									);
								},
							}
						},
						Err(error) => {
							warn!(
								"[ExtensionManagementService] Failed to read package.json at {:?}: {}",
								package_json_path, error
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
		info!("[ExtensionManagementService] Scanning for extensions...");

		// Get the extension scan paths from ApplicationState
		let ScanPaths:Vec<std::path::PathBuf> = {
			let ScanPathsGuard = self
				.ApplicationState
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
			.ScannedExtensions
			.lock()
			.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;

		ScannedExtensionsGuard.clear();

		for extension in extensions {
			if let Some(identifier) = extension.get("Identifier").and_then(|v| v.as_str()) {
				// Convert the extension DTO to ExtensionDescriptionStateDTO
				let extension_dto = ExtensionDescriptionStateDTO {
					Identifier:serde_json::Value::String(identifier.to_string()),
					Name:extension.get("Name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
					Version:extension.get("Version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string(),
					Publisher:extension
						.get("Publisher")
						.and_then(|v| v.as_str())
						.unwrap_or("Unknown")
						.to_string(),
					Engines:extension.get("Engines").cloned().unwrap_or(serde_json::Value::Null),
					Main:extension.get("Main").and_then(|v| v.as_str()).map(|s| s.to_string()),
					Browser:extension.get("Browser").and_then(|v| v.as_str()).map(|s| s.to_string()),
					ModuleType:extension.get("ModuleType").and_then(|v| v.as_str()).map(|s| s.to_string()),
					IsBuiltin:extension.get("IsBuiltin").and_then(|v| v.as_bool()).unwrap_or(false),
					IsUnderDevelopment:extension.get("IsUnderDevelopment").and_then(|v| v.as_bool()).unwrap_or(false),
					ExtensionLocation:extension.get("ExtensionLocation").cloned().unwrap_or(serde_json::Value::Null),
					ActivationEvents:extension
						.get("ActivationEvents")
						.and_then(|v| v.as_array())
						.map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
					Contributes:extension.get("Contributes").cloned(),
				};

				ScannedExtensionsGuard.insert(identifier.to_string(), extension_dto);
			}
		}

		info!("[ExtensionManagementService] Found {} extensions", ScannedExtensionsGuard.len());
		Ok(())
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

	async fn GetExtension(&self, id:String) -> Result<Option<Value>, CommonError> {
		let ScannedExtensionsGuard = self
			.ApplicationState
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

impl Requires<dyn WebviewProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WebviewProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn WorkspaceProvider> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WorkspaceProvider> { Arc::new(self.clone()) }
}

impl Requires<dyn WorkspaceEditApplier> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WorkspaceEditApplier> { Arc::new(self.clone()) }
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
