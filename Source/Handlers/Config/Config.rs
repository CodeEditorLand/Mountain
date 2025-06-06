// File: Handlers/Config/Config.rs
// Contains the primary logic for configuration management, including loading,
// merging, updating, and notifying about changes.

#![allow(non_snake_case, non_camel_case_types)]

use std::path::{Path, PathBuf};

use Common::{
	ConfigEffects::{ConfigurationTarget, IConfigurationOverrides},
	Errors::CommonError,
	IpcEffects::ProxyConfiguration as ProxyTarget,
};
use globset::GlobBuilder;
use log::{debug, error, info, trace, warn};
use serde_json::{Map, Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime};
use url::Url;

use crate::Vine; // The gRPC communication module
use crate::{
	AppState::{AppState, Dto::MergedConfigurationState},
	Handlers::ErrorUtils,
};

/// Maps a Mutex lock error to a `CommonError`.
fn MapAppStateLockErrorToCommonError<T>(Error:std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> CommonError {
	let ErrorMessage = format!("[ConfigHandler LockError] {}", Error);
	error!("{}", ErrorMessage);
	CommonError::StateLock { Context:ErrorMessage }
}

/// Retrieves the filesystem path for a specific configuration target (User,
/// Workspace, etc.).
pub fn GetConfigPathForTarget<R:TauriRuntime>(
	ApplicationHandle:&AppHandle<R>,
	AppStateInstance:&AppState,
	Target:ConfigurationTarget,
	Overrides:&IConfigurationOverrides,
	_ScopeToLanguage:bool, // Parameter kept for signature, but not used in path resolution
) -> Result<PathBuf, CommonError> {
	trace!(
		"[ConfigHandler GetPath] Resolving: Target={:?}, Overrides.Resource={:?}, Overrides.LangId={:?}",
		Target,
		Overrides.Resource.as_ref().and_then(|v| v.get("external")),
		Overrides.OverrideIdentifier
	);

	let PathResolver = ApplicationHandle.path_resolver();
	let BaseUserConfigDirectory = PathResolver.app_config_dir().ok_or_else(|| {
		CommonError::ConfigLoad { Description:"Cannot resolve app config directory for User settings".to_string() }
	})?;

	match Target {
		ConfigurationTarget::UserLocal | ConfigurationTarget::User => {
			Ok(BaseUserConfigDirectory.join("User").join("settings.json"))
		},
		ConfigurationTarget::Workspace => {
			let ConfigPathGuard = AppStateInstance
				.WorkspaceConfigurationPath
				.lock()
				.map_err(MapAppStateLockErrorToCommonError)?;
			ConfigPathGuard.as_ref().cloned().ok_or_else(|| {
				CommonError::ConfigUpdate {
					Key:"target".to_string(),
					Description:"No workspace configuration file loaded; cannot target WORKSPACE settings.".to_string(),
				}
			})
		},
		ConfigurationTarget::WorkspaceFolder => {
			let ResourceUriValue = Overrides.Resource.as_ref().ok_or_else(|| {
				CommonError::InvalidArg {
					ArgumentName:"Overrides.Resource".to_string(),
					Reason:"Missing resource URI for WORKSPACE_FOLDER target.".to_string(),
				}
			})?;
			// Simplified URI parsing logic from original file
			let ResourceUriString = ResourceUriValue.get("external").and_then(Value::as_str).unwrap_or("");
			let ResourceUri = Url::parse(ResourceUriString).map_err(|e| {
				CommonError::InvalidArg {
					ArgumentName:"Resource".to_string(),
					Reason:format!("Invalid resource URI in overrides: {}", e),
				}
			})?;
			let FoldersGuard = AppStateInstance
				.WorkspaceFolders
				.lock()
				.map_err(MapAppStateLockErrorToCommonError)?;
			let ContainingFolder = FoldersGuard
				.iter()
				.find(|Folder| {
					ResourceUri.scheme() == Folder.Uri.scheme() && ResourceUri.path().starts_with(Folder.Uri.path())
				})
				.ok_or_else(|| {
					CommonError::ConfigLoad {
						Description:format!("Resource URI '{}' not in any workspace folder.", ResourceUri),
					}
				})?;
			Ok(PathBuf::from(ContainingFolder.Uri.path()).join(".vscode").join("settings.json"))
		},
		_ => Err(CommonError::NotImplemented { FeatureName:format!("ConfigurationTarget::{:?}", Target) }),
	}
}

/// Loads a JSON file from a given path, returning an empty JSON object if it
/// doesn't exist.
pub async fn LoadJsonFileIfExistsOrDefault(Path:&Path) -> Result<Value, CommonError> {
	trace!("[ConfigHandler LoadJson] Attempting load from: {}", Path.display());
	match tokio::fs::read_to_string(Path).await {
		Ok(Content) => {
			if Content.trim().is_empty() {
				Ok(json!({}))
			} else {
				serde_json::from_str(&Content).map_err(|e| {
					CommonError::ConfigLoad { Description:format!("JSON parse failed for {}: {}", Path.display(), e) }
				})
			}
		},
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
		Err(e) => {
			Err(CommonError::ConfigLoad { Description:format!("File read failed for {}: {}", Path.display(), e) })
		},
	}
}

/// Writes a JSON value to a file, creating parent directories if necessary.
pub async fn WriteJsonFile(Path:&Path, ValueToWrite:&Value) -> Result<(), CommonError> {
	trace!("[ConfigHandler WriteJson] Writing to: {}", Path.display());
	let ParentDirectory = Path.parent().ok_or_else(|| {
		CommonError::ConfigUpdate {
			Key:Path.to_string_lossy().to_string(),
			Description:"Invalid path (no parent)".to_string(),
		}
	})?;
	if !tokio::fs::try_exists(ParentDirectory).await.map_err(|e| {
		CommonError::ConfigUpdate {
			Key:ParentDirectory.to_string_lossy().to_string(),
			Description:format!("Failed to check existence: {}", e),
		}
	})? {
		tokio::fs::create_dir_all(ParentDirectory).await.map_err(|e| {
			CommonError::ConfigUpdate {
				Key:ParentDirectory.to_string_lossy().to_string(),
				Description:format!("Failed to create directory: {}", e),
			}
		})?;
	}
	let Content = serde_json::to_string_pretty(ValueToWrite)?;
	tokio::fs::write(Path, Content).await.map_err(|e| {
		CommonError::ConfigUpdate {
			Key:Path.to_string_lossy().to_string(),
			Description:format!("Failed to write file: {}", e),
		}
	})?;
	info!("[ConfigHandler WriteJson] Wrote JSON to {}", Path.display());
	Ok(())
}

/// Updates a key within a JSON Value object based on a dot-separated path.
pub fn UpdateJsonValueAtPath(TargetValue:&mut Value, KeyPath:&str, ValueToSet:Value) {
	let mut CurrentNode = TargetValue;
	let Parts:Vec<&str> = KeyPath.split('.').collect();
	if Parts.is_empty() {
		return;
	}

	let LastPartIndex = Parts.len() - 1;
	for (Index, Part) in Parts.iter().enumerate() {
		if Index == LastPartIndex {
			if let Some(ObjectMap) = CurrentNode.as_object_mut() {
				if ValueToSet.is_null() {
					ObjectMap.remove(*Part);
				} else {
					ObjectMap.insert(Part.to_string(), ValueToSet);
				}
			}
			return;
		}
		if !CurrentNode.is_object() {
			*CurrentNode = json!({});
		}
		CurrentNode = CurrentNode.as_object_mut().unwrap().entry(*Part).or_insert_with(|| json!({}));
	}
}

/// Loads all configuration files and merges them into a single state.
pub async fn LoadAndMergeConfigurationsInternal<R:TauriRuntime>(
	ApplicationHandle:&AppHandle<R>,
	AppStateInstance:&AppState,
) -> Result<MergedConfigurationState, CommonError> {
	info!("[ConfigHandler Merge] Loading and merging all configurations...");
	let UserConfigPath = GetConfigPathForTarget(
		ApplicationHandle,
		AppStateInstance,
		ConfigurationTarget::User,
		&IConfigurationOverrides::default(),
		false,
	)?;
	let mut MergedConfigData = LoadJsonFileIfExistsOrDefault(&UserConfigPath).await?;

	if let Some(WorkspaceConfigPath) = AppStateInstance
		.WorkspaceConfigurationPath
		.lock()
		.map_err(MapAppStateLockErrorToCommonError)?
		.clone()
	{
		if let Some(Settings) = LoadJsonFileIfExistsOrDefault(&WorkspaceConfigPath).await?.get("settings") {
			MergeJsonValues(&mut MergedConfigData, Settings);
		}
	}

	for FolderState in AppStateInstance
		.WorkspaceFolders
		.lock()
		.map_err(MapAppStateLockErrorToCommonError)?
		.iter()
	{
		if FolderState.Uri.scheme() == "file" {
			let FolderSettingsPath = PathBuf::from(FolderState.Uri.path()).join(".vscode").join("settings.json");
			if tokio::fs::try_exists(&FolderSettingsPath).await.unwrap_or(false) {
				let FolderValues = LoadJsonFileIfExistsOrDefault(&FolderSettingsPath).await?;
				MergeJsonValues(&mut MergedConfigData, &FolderValues);
			}
		}
	}
	Ok(MergedConfigurationState::New(MergedConfigData))
}

/// Merges the `Source` JSON value into the `Target` JSON value.
fn MergeJsonValues(Target:&mut Value, Source:&Value) {
	if let (Some(TargetMap), Some(SourceMap)) = (Target.as_object_mut(), Source.as_object()) {
		for (Key, SourceValue) in SourceMap {
			let TargetValue = TargetMap.entry(Key.clone()).or_insert_with(|| SourceValue.clone());
			MergeJsonValues(TargetValue, SourceValue);
		}
	} else {
		*Target = Source.clone();
	}
}

/// Notifies the Cocoon sidecar about configuration changes.
pub async fn NotifyConfigChangedForKeys<R:TauriRuntime>(ApplicationHandle:&AppHandle<R>, AffectedKeys:Vec<String>) {
	if AffectedKeys.is_empty() {
		return;
	}
	info!("[ConfigHandler Notify] Notifying Cocoon of config change: {:?}", AffectedKeys);

	let FullMethodName = format!(
		"{}.$acceptConfigurationChanged",
		ProxyTarget::ExtHostConfiguration.GetTargetPrefix()
	);
	let AppStateInstance = ApplicationHandle.state::<AppState>();
	let ConfigInitDataValue = {
		let ConfigGuard = AppStateInstance
			.Configuration
			.lock()
			.expect("Config lock failed for notification");
		let ScopesForRpc = ConfigGuard
			.GetAllScopesForRpc()
			.into_iter()
			.map(|(Key, Scope)| (Key, json!(Scope)))
			.collect();
		json!({
			"effective": ConfigGuard.Data.clone(),
			// Other fields are stubbed as per original logic
			"defaults": { "contents": {} }, "user": { "contents": {} }, "workspace": { "contents": {} },
			"folders": [], "memory": { "contents": {} }, "policy": Value::Null,
			"configurationScopes": ScopesForRpc,
		})
	};
	let ChangeEventDto = json!({ "keys": AffectedKeys, "overrides": [] });
	let ParametersArray = json!([ConfigInitDataValue, ChangeEventDto]);

	if let Err(Error) = Vine::SendNotification("cocoon-main".to_string(), FullMethodName, ParametersArray).await {
		error!("[ConfigHandler Notify] Failed to send notification: {}", Error);
	}
}

/// Matches a document against a DocumentSelector DTO.
pub fn MatchDocumentSelector(SelectorValue:&Value, DocumentUri:&Url, LanguageIdentifier:&str) -> bool {
	if let Some(SelectorLanguageId) = SelectorValue.as_str() {
		return SelectorLanguageId == LanguageIdentifier || SelectorLanguageId == "*";
	}
	if let Some(SelectorArray) = SelectorValue.as_array() {
		return SelectorArray
			.iter()
			.any(|Filter| MatchDocumentSelector(Filter, DocumentUri, LanguageIdentifier));
	}
	if SelectorValue.is_object() {
		// Simplified logic for object filter matching
		if let Some(Lang) = SelectorValue.get("language").and_then(Value::as_str) {
			if !(Lang == LanguageIdentifier || Lang == "*") {
				return false;
			}
		}
		if let Some(Scheme) = SelectorValue.get("scheme").and_then(Value::as_str) {
			if !(Scheme == DocumentUri.scheme() || Scheme == "*") {
				return false;
			}
		}
		if let Some(Pattern) = SelectorValue.get("pattern").and_then(Value::as_str) {
			if GlobBuilder::new(Pattern)
				.build()
				.map_or(false, |g| g.compile_matcher().is_match(DocumentUri.path()))
			{
				return true;
			} else {
				return false;
			}
		}
		return true; // If we reach here, all provided filters matched
	}
	false
}
