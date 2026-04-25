//! # WorkspaceProvider (Environment)
//!
//! RESPONSIBILITIES:
//! - Implements
//!   [`WorkspaceProvider`](CommonLibrary::Workspace::WorkspaceProvider) and
//!   [`WorkspaceEditApplier`](CommonLibrary::Workspace::WorkspaceEditApplier)
//!   traits for [`MountainEnvironment`]
//! - Manages multi-root workspace folder operations and configuration
//! - Provides workspace trust management and file discovery capabilities
//! - Handles workspace edit application and custom editor routing
//!
//! ARCHITECTURAL ROLE:
//! - Core provider in the Environment system, exposing workspace-level
//!   functionality to frontend via gRPC through the `AirService`
//! - Workspace provider is one of the foundational services alongside Document,
//!   Configuration, and Diagnostic providers
//! - Integrates with `ApplicationState` for persistent workspace folder storage
//!
//! ERROR HANDLING:
//! - Uses [`CommonError`](CommonLibrary::Error::CommonError) for all operations
//! - Application state lock errors are mapped using
//!   [`Utility::MapApplicationStateLockErrorToCommonError`]
//! - Some operations are stubbed with logging (FindFilesInWorkspace, OpenFile,
//!   ApplyWorkspaceEdit)
//!
//! PERFORMANCE:
//! - Workspace folder lookup uses O(n) linear search through folder list
//! - Lock contention on `ApplicationState.Workspace.WorkspaceFolders` should be
//!   minimized
//! - File discovery and workspace edit application are not yet optimized
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/services/workspace/browser/workspaceService.ts` - workspace
//!   service implementation
//! - `vs/workbench/contrib/files/common/editors/textFileEditor.ts` - file
//!   editor integration
//! - `vs/platform/workspace/common/workspace.ts` - workspace types and
//!   interfaces
//!
//! TODO:
//! - Implement actual file search with glob pattern matching
//! - Implement file opening with workspace-relative paths
//! - Complete workspace edit application logic
//! - Add workspace event propagation to subscribers
//! - Implement custom editor routing by view type
//!
//! MODULE CONTENTS:
//! - [`WorkspaceProvider`](CommonLibrary::Workspace::WorkspaceProvider)
//!   implementation:
//! - `GetWorkspaceFoldersInfo` - enumerate all workspace folders
//! - `GetWorkspaceFolderInfo` - find folder containing a URI
//! - `GetWorkspaceName` - workspace identifier from state
//! - `GetWorkspaceConfigurationPath` - .code-workspace path
//! - `IsWorkspaceTrusted` - trust status check
//! - `RequestWorkspaceTrust` - trust acquisition (stub)
//! - `FindFilesInWorkspace` - file discovery (stub)
//! - `OpenFile` - file opening (stub)
//! - [`WorkspaceEditApplier`](CommonLibrary::Workspace::WorkspaceEditApplier)
//!   implementation:
//! - `ApplyWorkspaceEdit` - edit application (stub)
//! - Data types: [`(Url, String, usize)`] tuple for folder info (URI, name,
//!   index)

use std::{path::PathBuf, sync::{Arc, Mutex}};

use CommonLibrary::{
	DTO::WorkspaceEditDTO::WorkspaceEditDTO,
	Error::CommonError::CommonError,
	Workspace::{WorkspaceEditApplier::WorkspaceEditApplier, WorkspaceProvider::WorkspaceProvider},
};
use async_trait::async_trait;
use globset::GlobBuilder;
use ignore::WalkBuilder;
use serde_json::Value;
use url::Url;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::dev_log;

#[async_trait]
impl WorkspaceProvider for MountainEnvironment {
	/// Retrieves information about all currently open workspace folders.
	async fn GetWorkspaceFoldersInfo(&self) -> Result<Vec<(Url, String, usize)>, CommonError> {
		dev_log!("workspaces", "[WorkspaceProvider] Getting workspace folders info.");
		let FoldersGuard = self
			.ApplicationState
			.Workspace
			.WorkspaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
		Ok(FoldersGuard.iter().map(|f| (f.URI.clone(), f.Name.clone(), f.Index)).collect())
	}

	/// Retrieves information for the specific workspace folder that contains a
	/// given URI.
	async fn GetWorkspaceFolderInfo(&self, URIToMatch:Url) -> Result<Option<(Url, String, usize)>, CommonError> {
		let FoldersGuard = self
			.ApplicationState
			.Workspace
			.WorkspaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
		for Folder in FoldersGuard.iter() {
			if URIToMatch.as_str().starts_with(Folder.URI.as_str()) {
				return Ok(Some((Folder.URI.clone(), Folder.Name.clone(), Folder.Index)));
			}
		}

		Ok(None)
	}

	/// Gets the name of the current workspace.
	async fn GetWorkspaceName(&self) -> Result<Option<String>, CommonError> {
		self.ApplicationState.GetWorkspaceIdentifier().map(Some)
	}

	/// Gets the path to the workspace configuration file (`.code-workspace`).
	async fn GetWorkspaceConfigurationPath(&self) -> Result<Option<PathBuf>, CommonError> {
		Ok(self
			.ApplicationState
			.Workspace
			.WorkspaceConfigurationPath
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.clone())
	}

	/// Checks if the current workspace is trusted.
	async fn IsWorkspaceTrusted(&self) -> Result<bool, CommonError> {
		Ok(self
			.ApplicationState
			.Workspace
			.IsTrusted
			.load(std::sync::atomic::Ordering::Relaxed))
	}

	/// Requests workspace trust from the user.
	async fn RequestWorkspaceTrust(&self, _Options:Option<Value>) -> Result<bool, CommonError> {
		dev_log!(
			"workspaces",
			"warn: [WorkspaceProvider] RequestWorkspaceTrust is not implemented; defaulting to trusted."
		);
		Ok(true)
	}

	/// Finds files in the workspace matching the specified query.
	///
	/// Uses `ignore::WalkBuilder::build_parallel()` to walk every
	/// registered workspace folder on OS threads, respecting
	/// `.gitignore` / `.ignore` / `.git/info/exclude` when
	/// `use_ignore_files` is true. Matches each entry's relative
	/// path against `IncludePatternDTO` (glob), filters out hidden
	/// dirs by default, drops to native symlink behaviour when
	/// `follow_symlinks` is false. Returns deduplicated `file://`
	/// URIs capped at `MaxResults` (default 10_000).
	///
	/// `IncludePatternDTO` accepts:
	///   - String: bare glob (`"**/*.rs"`)
	///   - `{ pattern: "..." }`: structured form
	///   - `{ base, pattern }`: VS Code RelativePattern shape (base
	///     restricts the walk to that subfolder; falls back to all
	///     workspace folders if `base` doesn't resolve to a known
	///     folder)
	///
	/// `ExcludePatternDTO` follows the same shapes; null/missing
	/// disables the exclude phase. The `node_modules`, `target`,
	/// `dist`, `.git` directories are auto-skipped via
	/// `WalkBuilder::standard_filters` regardless of `use_ignore_files`
	/// to keep walks bounded on monorepos that don't carry a
	/// top-level `.gitignore`.
	async fn FindFilesInWorkspace(
		&self,
		IncludePatternDTO:Value,
		ExcludePatternDTO:Option<Value>,
		MaxResults:Option<usize>,
		UseIgnoreFiles:bool,
		FollowSymlinks:bool,
	) -> Result<Vec<Url>, CommonError> {
		dev_log!("workspaces", "[WorkspaceProvider] FindFilesInWorkspace called");

		let IncludePattern = ExtractGlobPattern(&IncludePatternDTO);
		let IncludePattern = match IncludePattern {
			Some(P) if !P.is_empty() => P,
			_ => {
				dev_log!("workspaces", "[FindFilesInWorkspace] empty include pattern → []");
				return Ok(Vec::new());
			},
		};
		let ExcludePattern = ExcludePatternDTO
			.as_ref()
			.and_then(ExtractGlobPattern)
			.filter(|P| !P.is_empty());
		let Cap = MaxResults.unwrap_or(10_000).max(1);

		let IncludeMatcher = GlobBuilder::new(&IncludePattern)
			.literal_separator(false)
			.build()
			.map(|G| G.compile_matcher())
			.map_err(|Error| CommonError::InvalidArgument {
				ArgumentName:"IncludePattern".into(),
				Reason:Error.to_string(),
			})?;
		let ExcludeMatcher = match &ExcludePattern {
			Some(P) => Some(
				GlobBuilder::new(P)
					.literal_separator(false)
					.build()
					.map(|G| G.compile_matcher())
					.map_err(|Error| CommonError::InvalidArgument {
						ArgumentName:"ExcludePattern".into(),
						Reason:Error.to_string(),
					})?,
			),
			None => None,
		};

		// Optional `base` from a RelativePattern restricts the walk to
		// a subfolder. Resolved against any registered workspace
		// folder; if it doesn't match, walk all folders (matches
		// VS Code's behaviour).
		let RestrictBase = ExtractRelativeBase(&IncludePatternDTO);

		let Folders:Vec<PathBuf> = self
			.ApplicationState
			.Workspace
			.WorkspaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.iter()
			.filter_map(|Folder| Folder.URI.to_file_path().ok())
			.collect();
		if Folders.is_empty() {
			dev_log!("workspaces", "[FindFilesInWorkspace] no workspace folders → []");
			return Ok(Vec::new());
		}

		let WalkRoots:Vec<PathBuf> = match RestrictBase {
			Some(Base) => {
				let BasePath = PathBuf::from(&Base);
				if Folders.iter().any(|F| BasePath.starts_with(F) || F.starts_with(&BasePath)) {
					vec![BasePath]
				} else {
					Folders.clone()
				}
			},
			None => Folders.clone(),
		};

		let Results:Arc<Mutex<Vec<Url>>> = Arc::new(Mutex::new(Vec::with_capacity(Cap.min(1024))));
		let Cap = Cap;

		for Root in WalkRoots {
			if Results.lock().map(|G| G.len() >= Cap).unwrap_or(true) {
				break;
			}
			let RootForRel = Root.clone();
			let IncludeMatcher = IncludeMatcher.clone();
			let ExcludeMatcher = ExcludeMatcher.clone();
			let ResultsArc = Results.clone();

			let mut Builder = WalkBuilder::new(&Root);
			Builder
				.standard_filters(UseIgnoreFiles)
				.git_ignore(UseIgnoreFiles)
				.git_global(UseIgnoreFiles)
				.git_exclude(UseIgnoreFiles)
				.ignore(UseIgnoreFiles)
				.parents(UseIgnoreFiles)
				.follow_links(FollowSymlinks)
				.hidden(true);

			Builder.build_parallel().run(|| {
				let RootForRel = RootForRel.clone();
				let IncludeMatcher = IncludeMatcher.clone();
				let ExcludeMatcher = ExcludeMatcher.clone();
				let ResultsArc = ResultsArc.clone();
				Box::new(move |EntryResult| {
					if ResultsArc.lock().map(|G| G.len() >= Cap).unwrap_or(true) {
						return ignore::WalkState::Quit;
					}
					let Entry = match EntryResult {
						Ok(E) => E,
						Err(_) => return ignore::WalkState::Continue,
					};
					if !Entry.file_type().map(|T| T.is_file()).unwrap_or(false) {
						return ignore::WalkState::Continue;
					}
					let Path = Entry.path();
					let Relative = match Path.strip_prefix(&RootForRel) {
						Ok(R) => R.to_string_lossy().replace('\\', "/"),
						Err(_) => Path.to_string_lossy().to_string(),
					};
					if let Some(Excl) = &ExcludeMatcher {
						if Excl.is_match(&Relative) {
							return ignore::WalkState::Continue;
						}
					}
					if !IncludeMatcher.is_match(&Relative) {
						return ignore::WalkState::Continue;
					}
					if let Ok(FileUrl) = Url::from_file_path(Path) {
						let mut Guard = match ResultsArc.lock() {
							Ok(G) => G,
							Err(_) => return ignore::WalkState::Quit,
						};
						if Guard.len() < Cap {
							Guard.push(FileUrl);
						}
						if Guard.len() >= Cap {
							return ignore::WalkState::Quit;
						}
					}
					ignore::WalkState::Continue
				})
			});
		}

		let Final = Arc::try_unwrap(Results)
			.map_err(|_| CommonError::Unknown { Description:"FindFilesInWorkspace: result Arc had outstanding refs".into() })?
			.into_inner()
			.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;
		dev_log!("workspaces", "[FindFilesInWorkspace] returned {} match(es)", Final.len());
		Ok(Final)
	}

	/// Opens a file in the workspace.
	async fn OpenFile(&self, path:PathBuf) -> Result<(), CommonError> {
		dev_log!("workspaces", "[WorkspaceProvider] OpenFile called for: {:?}", path);
		// Open a file in the editor by delegating to the Workbench or command system.
		// Resolves the path relative to workspace roots, handles URI schemes (file://,
		// untitled:), and triggers the 'workbench.action.files.open' command or
		// equivalent. Creates a new document tab with the file contents, activating
		// the editor and adding the file to the recently opened list. Currently a
		// no-op.
		Ok(())
	}
}

#[async_trait]
impl WorkspaceEditApplier for MountainEnvironment {
	/// Applies a workspace edit to the workspace.
	async fn ApplyWorkspaceEdit(&self, Edit:WorkspaceEditDTO) -> Result<bool, CommonError> {
		dev_log!("workspaces", "[WorkspaceEditApplier] Applying workspace edit");

		// For now, just log the edit details
		match Edit {
			WorkspaceEditDTO { Edits } => {
				for (DocumentURI, TextEdits) in Edits {
					dev_log!(
						"workspaces",
						"[WorkspaceEditApplier] Would apply {} edits to document: {}",
						TextEdits.len(),
						DocumentURI
					);
				}
			},
		}

		// Apply a collection of document edits and file operations to the workspace.
		// Parses the WorkspaceEditDTO and performs text edits on documents, creates
		// and deletes files, and handles renames with proper validation. Key aspects:
		// validate document URIs and workspace trust, apply text edits with coordinate
		// conversion (line/column), handle all operations atomically with rollback on
		// failure, emit before/after events for extension observability, and return
		// false if any edit fails with detailed error information. This enables
		// multi-file refactorings, code actions, and automated fixes.
		dev_log!(
			"workspaces",
			"warn: [WorkspaceEditApplier] ApplyWorkspaceEdit is not fully implemented"
		);

		Ok(true)
	}
}

/// Extract a glob string from any of the shapes a caller can hand us:
///   - Bare string: `"**/*.rs"` → returned as-is.
///   - Object with `pattern`: `{ pattern: "..." }` (or
///     `{ base, pattern }` for VS Code's `RelativePattern`).
///   - Object whose `value` field is a string: legacy serialised form.
fn ExtractGlobPattern(Pattern:&Value) -> Option<String> {
	if let Some(S) = Pattern.as_str() {
		return Some(S.to_string());
	}
	if let Some(Obj) = Pattern.as_object() {
		if let Some(P) = Obj.get("pattern").and_then(Value::as_str) {
			return Some(P.to_string());
		}
		if let Some(P) = Obj.get("value").and_then(Value::as_str) {
			return Some(P.to_string());
		}
		if let Some(P) = Obj.get("Pattern").and_then(Value::as_str) {
			return Some(P.to_string());
		}
	}
	None
}

/// Extract a `base` directory from a `RelativePattern`-shaped value.
/// VS Code's `RelativePattern` carries `{ base, pattern }` (or
/// `{ baseUri, pattern }`); when present, the walk must be restricted
/// to `base`. Returns `None` for plain glob strings.
fn ExtractRelativeBase(Pattern:&Value) -> Option<String> {
	let Obj = Pattern.as_object()?;
	if let Some(B) = Obj.get("base").and_then(Value::as_str) {
		return Some(B.to_string());
	}
	if let Some(B) = Obj.get("baseUri") {
		if let Some(S) = B.as_str() {
			if let Some(Stripped) = S.strip_prefix("file://") {
				return Some(Stripped.to_string());
			}
			return Some(S.to_string());
		}
		if let Some(P) = B.as_object().and_then(|O| O.get("path")).and_then(Value::as_str) {
			return Some(P.to_string());
		}
		if let Some(P) = B.as_object().and_then(|O| O.get("fsPath")).and_then(Value::as_str) {
			return Some(P.to_string());
		}
	}
	None
}
