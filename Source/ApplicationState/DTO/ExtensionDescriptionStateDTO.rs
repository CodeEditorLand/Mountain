//! # ExtensionDescriptionStateDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for extension description/state
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to track extension metadata and capabilities
//!
//! # FIELDS
//! - Identifier: Unique extension identifier
//! - Name: Extension display name
//! - Version: Semantic version string
//! - Publisher: Publisher name
//! - Engines: Engine compatibility requirements
//! - Main: Main entry point path (Node.js)
//! - Browser: Browser entry point path
//! - ModuleType: Module type (commonjs/esm)
//! - IsBuiltin: Built-in extension flag
//! - IsUnderDevelopment: Development flag
//! - ExtensionLocation: Installation location URI
//! - ActivationEvents: Activation event triggers
//! - Contributes: Extension contributions configuration

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Maximum length for extension name
const MAX_EXTENSION_NAME_LENGTH:usize = 128;

/// Maximum length for version string
const MAX_VERSION_LENGTH:usize = 64;

/// Maximum length for publisher name
const MAX_PUBLISHER_LENGTH:usize = 64;

/// Maximum number of activation events
const MAX_ACTIVATION_EVENTS:usize = 100;

/// the deserialized content of an extension's `package.json` file,
/// augmented with location information and other metadata.
/// This is stored in `ApplicationState` to provide the extension host with the
/// list of available extensions and their capabilities.
/// VS Code extensions use camelCase in package.json. Serde renames from
/// PascalCase Rust fields to camelCase JSON automatically. Fields that
/// don't exist in package.json (Identifier, ExtensionLocation, IsBuiltin)
/// default to their zero values on deserialization.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionDescriptionStateDTO {
	// --- Core Metadata ---
	/// Extension identifier: { value: string, uuid?: string }
	/// Not present in package.json - constructed from publisher.name after
	/// parsing.
	#[serde(default)]
	pub Identifier:Value,

	/// Extension name (from package.json "name")
	/// Always serialized, even when empty, because VS Code's scanner and
	/// the trusted-publishers migration (`extensions.contribution.ts`) both
	/// evaluate `extension.manifest.name.toLowerCase()` unconditionally.
	/// Dropping the field would leave a bare `undefined` and crash the
	/// renderer with `TypeError: undefined is not an object`.
	#[serde(default)]
	pub Name:String,

	/// Semantic version string (e.g., "1.0.0").
	/// Always serialized for the same reason as `Name` / `Publisher`: the
	/// renderer reads `manifest.version` in several hot paths and crashes
	/// if the field is missing outright.
	#[serde(default)]
	pub Version:String,

	/// Publisher name or identifier.
	/// Always serialized, even when empty. VS Code's
	/// `extensions.contribution.ts` trusted-publishers migration runs on
	/// every User-extension at workbench boot and executes
	/// `extension.manifest.publisher.toLowerCase()`. If the key is omitted
	/// the renderer crashes with
	/// `TypeError: undefined is not an object (evaluating
	/// 'manifest.publisher')`.
	#[serde(default)]
	pub Publisher:String,

	/// Engine compatibility requirements: { vscode: string }
	#[serde(default)]
	pub Engines:Value,

	// --- Entry Points ---
	/// Main entry point path (Node.js runtime)
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Main:Option<String>,

	/// Browser entry point path (web extension)
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Browser:Option<String>,

	// --- Type & Flags ---
	/// Module type: commonjs or esm
	#[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
	pub ModuleType:Option<String>,

	/// Whether this is a built-in extension (not in package.json, set by
	/// scanner)
	#[serde(default)]
	pub IsBuiltin:bool,

	/// Whether extension is under active development
	#[serde(default)]
	pub IsUnderDevelopment:bool,

	// --- Location & Activation ---
	/// Installation location URI (set by scanner, not in package.json)
	#[serde(default)]
	pub ExtensionLocation:Value,

	/// Activation event triggers (e.g., "onStartupFinished")
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ActivationEvents:Option<Vec<String>>,

	// --- Contributions ---
	/// Extension contributions (commands, views, etc.)
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Contributes:Option<Value>,

	// --- Discovery metadata ---
	/// VS Code category tags ("Themes", "Programming Languages",
	/// "Snippets", "Language Packs", "Debuggers", "Formatters",
	/// "Keymaps", "SCM Providers", "Testing", "Education", "Other").
	/// Atom TH1: Wind's Extensions sidebar filters `@builtin category:themes`
	/// against this array. Without the field the filter never matches -
	/// user reported theme extensions absent on @builtin search despite
	/// being on disk. Scanner-passthrough surfaces the raw package.json
	/// value; resolved NLS placeholders survive because the serde
	/// deserialisation happens after the NLS rewrite.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Categories:Option<Vec<String>>,

	/// Human-readable display name, usually a VS Code NLS placeholder like
	/// `%displayName%` that the scanner resolves against `package.nls.json`.
	/// Atom TH1: sidebar row rendering falls back to `name` when this is
	/// absent; having the real value populates tooltips and the
	/// details editor.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub DisplayName:Option<String>,

	/// Short prose description. Same NLS-placeholder rules as DisplayName.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Description:Option<String>,

	/// Extension keywords array - searched by the sidebar when the query
	/// doesn't match `name`, `displayName`, or `description`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Keywords:Option<Vec<String>>,

	/// Repository info: Either a string URL or `{ type, url }` object.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Repository:Option<Value>,

	/// Bug-tracker URL or object.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Bugs:Option<Value>,

	/// Homepage URL.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Homepage:Option<String>,

	/// License identifier (SPDX short code) or URL.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub License:Option<String>,

	/// Icon path relative to the extension root (for sidebar thumbnails).
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Icon:Option<String>,

	/// Marketplace API key placeholder - still present in some upstream
	/// built-in manifests. `@vscode/extension-telemetry` reads its
	/// length on construction; if missing the activate throws
	/// `Cannot read properties of undefined (reading 'length')`.
	#[serde(default, skip_serializing_if = "Option::is_none", rename = "aiKey")]
	pub AiKey:Option<String>,

	/// Marketplace-side extension kind (`["ui"]`, `["workspace"]`,
	/// `["web"]`). Wind uses this to decide which host to run the
	/// extension in. Missing → VS Code falls back to heuristics.
	#[serde(default, skip_serializing_if = "Option::is_none", rename = "extensionKind")]
	pub ExtensionKind:Option<Value>,

	/// Capabilities descriptor - `untrustedWorkspaces`, `virtualWorkspaces`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Capabilities:Option<Value>,

	/// Dependency list - other extensions this one needs activated first.
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		rename = "extensionDependencies"
	)]
	pub ExtensionDependencies:Option<Vec<String>>,

	/// Extension-pack children - extensions this one bundles by reference.
	#[serde(default, skip_serializing_if = "Option::is_none", rename = "extensionPack")]
	pub ExtensionPack:Option<Vec<String>>,
}

impl ExtensionDescriptionStateDTO {
	/// Validates the extension description data.
	/// # Returns
	/// Result indicating success or validation error with reason
	pub fn Validate(&self) -> Result<(), String> {
		// Validate Name length
		if self.Name.len() > MAX_EXTENSION_NAME_LENGTH {
			return Err(format!(
				"Extension name exceeds maximum length of {} bytes",
				MAX_EXTENSION_NAME_LENGTH
			));
		}

		// Validate Version length
		if self.Version.len() > MAX_VERSION_LENGTH {
			return Err(format!("Version string exceeds maximum length of {} bytes", MAX_VERSION_LENGTH));
		}

		// Validate Publisher length
		if self.Publisher.len() > MAX_PUBLISHER_LENGTH {
			return Err(format!("Publisher exceeds maximum length of {} bytes", MAX_PUBLISHER_LENGTH));
		}

		// Validate ActivationEvents count
		if let Some(Events) = &self.ActivationEvents {
			if Events.len() > MAX_ACTIVATION_EVENTS {
				return Err(format!("Activation events exceed maximum count of {}", MAX_ACTIVATION_EVENTS));
			}
		}

		Ok(())
	}

	/// Creates a minimal extension description for testing or placeholder use.
	/// # Arguments
	/// * `Identifier` - Extension identifier value
	/// * `Name` - Extension name
	/// * `Version` - Extension version
	/// * `Publisher` - Publisher name
	/// # Returns
	/// A new ExtensionDescriptionStateDTO with minimal required fields
	pub fn CreateMinimal(Identifier:Value, Name:String, Version:String, Publisher:String) -> Result<Self, String> {
		let Description = Self {
			Identifier,

			Name:Name.clone(),

			Version:Version.clone(),

			Publisher:Publisher.clone(),

			Engines:serde_json::json!({ "vscode": "*" }),

			Main:None,

			Browser:None,

			ModuleType:None,

			IsBuiltin:false,

			IsUnderDevelopment:false,

			ExtensionLocation:serde_json::json!(null),

			ActivationEvents:None,

			Contributes:None,

			Categories:None,

			DisplayName:None,

			Description:None,

			Keywords:None,

			Repository:None,

			Bugs:None,

			Homepage:None,

			License:None,

			Icon:None,

			AiKey:None,

			ExtensionKind:None,

			Capabilities:None,

			ExtensionDependencies:None,

			ExtensionPack:None,
		};

		Description.Validate()?;

		Ok(Description)
	}
}
