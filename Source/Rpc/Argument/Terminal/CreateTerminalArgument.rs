

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value; // For fields like Argument, Cwd, IconPath, Location which can be complex

// This DTO represents the options for creating a new terminal instance.
// It mirrors the structure of VS Code's `TerminalOptions` and
// `ExtensionTerminalOptions`.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CreateTerminalArgument {
	// Optional human-readable name for the terminal.
	pub Name:Option<String>,
	// Optional path to the shell executable. If not provided, a default shell is used.
	#[serde(alias = "executable", alias = "shellPath")] // Common aliases
	pub ShellPath: Option<String>,
	// Optional arguments for the shell. Can be a string or an array of strings.
	#[serde(alias = "args", alias = "shellArgument")] // Common aliases
	pub ShellArgument: Option<Value>,
	// Optional current working directory for the terminal. Can be a string path or a UriComponents DTO.
	#[serde(alias = "cwd")]
	pub CurrentWorkingDirectory:Option<Value>,
	// Optional environment variables to set for the terminal process.
	// Values can be strings or null (to unset).
	#[serde(alias = "env")]
	pub EnvironmentVariables:Option<HashMap<String, Option<String>>>,
	// If true, the terminal will not inherit the environment of the parent process.
	#[serde(alias = "strictEnv")]
	pub StrictEnvironment:Option<bool>,
	// If true, the terminal will not be shown in the UI by default.
	#[serde(alias = "hideFromUser")]
	pub HideFromUser:Option<bool>,
	// If true, the terminal is transient and may not be persisted across sessions.
	#[serde(alias = "isTransient")]
	pub IsTransient:Option<bool>,
	// Optional icon for the terminal. Can be a UriComponents DTO or a ThemeIcon DTO.
	#[serde(alias = "iconPath")]
	pub IconPath:Option<Value>,
	// Optional theme color identifier for the terminal icon.
	pub Color:Option<String>,
	// Optional initial text to send to the terminal upon creation.
	#[serde(alias = "initialText")]
	pub InitialText:Option<String>,
	// Defines behavior when the shell process exits. (e.g., "never", "always", "onError")
	#[serde(alias = "waitOnExit")]
	pub WaitOnExit:Option<Value>, // Could be a string or a more complex DTO
	// Identifier used by the extension host for this terminal.
	#[serde(alias = "extHostTerminalId")]
	pub ExtensionHostTerminalIdentifier:Option<String>,
	// If true, a PTY (pseudo-terminal) should be used. Defaults to true.
	#[serde(alias = "isPty")]
	pub IsPty:Option<bool>,
	// Specifies where the terminal should be shown (e.g., view column, split behavior).
	// Can be a number (ViewColumn) or a more complex object.
	pub Location:Option<Value>,

	// --- Fields typically derived or used internally by VS Code ---
	#[serde(alias = "cwdIsResolved")]
	pub CurrentWorkingDirectoryIsResolved:Option<bool>,
	#[serde(alias = "isFeatureTerminal")]
	pub IsFeatureTerminal:Option<bool>,
	#[serde(alias = "useShellEnvironment")]
	pub UseShellEnvironment:Option<bool>,
	#[serde(alias = "isUserInitiated")]
	pub IsUserInitiated:Option<bool>,
	// A nonce used for PTY security if applicable.
	#[serde(alias = "ptyNonce")]
	pub PtyNonce:Option<String>,
}

// Helper to get initial dimensions, as it was in the original struct but not
// directly in options. This is a conceptual addition if the `location` field
// doesn't directly carry this. If `location` (a Value) can contain
// `initialDimensions`, this helper might be less needed.
impl CreateTerminalArgument {
	pub fn GetInitialDimensionsFromOptions(&self) -> Option<Value> {
		if let Some(LocationValue) = &self.Location {
			if let Some(DimensionsValue) = LocationValue.get("initialDimensions") {
				return Some(DimensionsValue.clone());
			}
			// VS Code sometimes has rows/cols directly on the location object for split
			// terminals.
			if LocationValue.get("rows").is_some() && LocationValue.get("cols").is_some() {
				return Some(LocationValue.clone());
			}
		}
		None
	}
}
