
// Defines the primary Data Transfer Objects (DTOs) used for initializing the
// Sky (frontend) environment. These structures are serialized to JSON and sent
// to the frontend to provide it with necessary bootstrap information.

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, path::PathBuf};

use serde::Serialize;
use url::Url;

/// Contains version information about the application and its components.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ProcessVersionsDto {
	pub AppName:Option<String>,
	pub AppVersion:Option<String>,
	pub TauriVersion:Option<String>,
	pub WebviewRuntimeVersion:Option<String>,
}

/// Contains information for Natural Language Support (NLS), i.e., localization.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NlsConfigurationDto {
	#[serde(alias = "messages")]
	pub MessageMap:HashMap<String, String>,
	pub Language:String,
	#[serde(alias = "availableLanguages")]
	pub AvailableLanguageMap:HashMap<String, String>,
	pub Pseudo:Option<bool>,
}

/// Contains product-specific information like names, version, and branding
/// identifiers.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ProductConfigurationDto {
	pub NameShort:Option<String>,
	pub NameLong:Option<String>,
	pub ApplicationName:Option<String>,
	pub Version:Option<String>,
	pub Commit:Option<String>,
	pub Date:Option<String>,
	pub DataFolderName:Option<String>,
	pub EmbedderIdentifier:Option<String>,
	#[serde(flatten)]
	pub AdditionalProperties:HashMap<String, serde_json::Value>,
}

/// The main configuration object sent to the frontend to bootstrap its
/// environment. This aggregates all necessary initial state information.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SandboxConfigurationDto {
	#[serde(alias = "windowId")]
	pub WindowIdentifier:String,
	#[serde(alias = "machineId")]
	pub MachineIdentifier:String,
	#[serde(alias = "sessionId")]
	pub SessionIdentifier:String,
	#[serde(alias = "sqmId")]
	pub SqmIdentifier:Option<String>,
	pub LogLevel:u32,
	#[serde(alias = "userEnv")]
	pub UserEnvironment:HashMap<String, Option<String>>,
	pub AppRoot:String,
	pub AppName:String,
	pub AppUriScheme:String,
	pub AppLanguage:String,
	pub AppHost:String,
	pub ProductQuality:Option<String>,
	pub Platform:String,
	#[serde(alias = "arch")]
	pub Architecture:String,
	pub Versions:ProcessVersionsDto,
	#[serde(alias = "execPath")]
	pub ExecutablePath:String,
	pub ZoomLevel:Option<f64>,
	#[serde(alias = "homeDir")]
	pub HomeDirectory:String,
	#[serde(alias = "tmpDir")]
	pub TemporaryDirectory:String,
	#[serde(alias = "userDataDir")]
	pub UserDataDirectory:String,
	pub BackupPath:Option<String>,
	#[serde(alias = "crashReporterId")]
	pub CrashReporterIdentifier:Option<String>,
	#[serde(alias = "nls")]
	pub Nls:NlsConfigurationDto,
	#[serde(alias = "productConfiguration")]
	pub ProductConfiguration:ProductConfigurationDto,
	#[serde(rename = "VSCODE_CWD")]
	pub VsCodeCurrentWorkingDirectory:Option<String>,
	pub ResourcesPath:String,
	#[serde(flatten)]
	pub AdditionalProperties:HashMap<String, serde_json::Value>,
}
