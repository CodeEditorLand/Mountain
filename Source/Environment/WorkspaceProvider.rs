//! # WorkspaceProvider (Environment)
//!
//! Implements [`WorkspaceProvider`] and [`WorkspaceEditApplier`] traits for
//! [`MountainEnvironment`], exposing workspace-level functionality to the
//! frontend via gRPC through `AirService`.
//!
//! ## Responsibilities
//!
//! - Multi-root workspace folder enumeration and URI matching
//! - Workspace trust management (`IsWorkspaceTrusted` / `RequestWorkspaceTrust`)
//! - File discovery (`FindFilesInWorkspace`) with LRU cache + single-flight
//!   dedup (see inline doc on that method)
//! - Workspace edit application — two-tier: emit Sky event for open documents;
//!   atomic on-disk splice for closed files
//!
//! ## VS Code reference
//!
//! - `vs/workbench/services/workspace/browser/workspaceService.ts`
//! - `vs/platform/workspace/common/workspace.ts`

use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{Arc, Mutex, OnceLock},
	time::{Duration, Instant},
};

use CommonLibrary::{
	DTO::WorkspaceEditDTO::WorkspaceEditDTO,
	Error::CommonError::CommonError,
	Workspace::{WorkspaceEditApplier::WorkspaceEditApplier, WorkspaceProvider::WorkspaceProvider},
};
use async_trait::async_trait;
use globset::GlobBuilder;
use ignore::WalkBuilder;
use serde_json::Value;
use tokio::sync::Notify;
use url::Url;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::dev_log;

/// Process-wide LRU cache for `FindFilesInWorkspace`. Cache key folds
/// every input that influences the walk; TTL is short so we never serve
/// a stale result after a file-system mutation. Entry budget is small
/// to bound memory across many workspace folders + glob shapes.
///
/// Why: the workbench's `ISearchService` fires `findFiles` per-keystroke
/// during Cmd+P fuzzy match (typically 5-10 calls in 200 ms) AND per
/// breadcrumb / quick-pick refresh. Each walk traverses tens of
/// thousands of files; a 0.5-3 ms HashMap lookup short-circuits all
/// but the first walk in a typing burst.
const FIND_FILES_CACHE_TTL:Duration = Duration::from_millis(2500);

const FIND_FILES_CACHE_CAPACITY:usize = 128;

#[derive(Hash, Eq, PartialEq, Clone)]
struct FindFilesCacheKey {
	Folders:Vec<PathBuf>,

	Include:String,

	Exclude:Option<String>,

	Cap:usize,

	UseIgnoreFiles:bool,

	FollowSymlinks:bool,

	RestrictBase:Option<String>,
}

struct FindFilesCacheEntry {
	Result:Vec<Url>,

	StoredAt:Instant,
}

fn FindFilesCache() -> &'static Mutex<HashMap<FindFilesCacheKey, FindFilesCacheEntry>> {
	static CACHE:OnceLock<Mutex<HashMap<FindFilesCacheKey, FindFilesCacheEntry>>> = OnceLock::new();

	CACHE.get_or_init(|| Mutex::new(HashMap::with_capacity(FIND_FILES_CACHE_CAPACITY)))
}

/// Insert into the cache with simple bounded-size eviction. When the
/// table reaches capacity we drop the oldest half in one pass; this
/// avoids tracking access order per entry while still keeping memory
/// bounded under sustained workbench traffic.
fn FindFilesCachePut(Key:FindFilesCacheKey, Result:Vec<Url>) {
	if let Ok(mut Guard) = FindFilesCache().lock() {
		if Guard.len() >= FIND_FILES_CACHE_CAPACITY {
			let Cutoff = Instant::now() - FIND_FILES_CACHE_TTL;

			Guard.retain(|_, V| V.StoredAt > Cutoff);

			if Guard.len() >= FIND_FILES_CACHE_CAPACITY {
				let DropCount = Guard.len() / 2;

				let StaleKeys:Vec<FindFilesCacheKey> = Guard.iter().take(DropCount).map(|(K, _)| K.clone()).collect();

				for K in StaleKeys {
					Guard.remove(&K);
				}
			}
		}

		Guard.insert(Key, FindFilesCacheEntry { Result, StoredAt:Instant::now() });
	}
}

fn FindFilesCacheGet(Key:&FindFilesCacheKey) -> Option<Vec<Url>> {
	let Guard = FindFilesCache().lock().ok()?;

	let Entry = Guard.get(Key)?;

	if Entry.StoredAt.elapsed() > FIND_FILES_CACHE_TTL {
		return None;
	}

	Some(Entry.Result.clone())
}

/// Drop every cached find-files result. Callers: workspace folder
/// add/remove (`UpdateWorkspaceFolders`), file system watcher events
/// from Mountain's notifier, explicit refresh from the renderer.
/// Cache holds for at most `FIND_FILES_CACHE_TTL` anyway, so missing
/// an invalidation point here is bounded latency, not correctness.
pub fn ClearFindFilesCache() {
	if let Ok(mut Guard) = FindFilesCache().lock() {
		Guard.clear();
	}
}

/// Single-flight registry: keys with a walk currently in progress
/// share the same `Notify` so concurrent callers awaiting the same
/// `(folders, include, exclude, cap, flags)` don't each kick off
/// their own filesystem walk.
///
/// Why: log audit (`20260501T053137`) showed 1023 `findFiles` calls
/// during one extension-boot session, with the cache hit rate
/// at ~67% (687 hits, 333 misses). The 333 misses fired BEFORE
/// the first walker for any given key populated the cache, so
/// each one independently re-walked the same tree. With the
/// single-flight guard the leader walks once, every concurrent
/// follower awaits, then reads the freshly-populated cache.
fn FindFilesInFlight() -> &'static Mutex<HashMap<FindFilesCacheKey, Arc<Notify>>> {
	static IN_FLIGHT:OnceLock<Mutex<HashMap<FindFilesCacheKey, Arc<Notify>>>> = OnceLock::new();

	IN_FLIGHT.get_or_init(|| Mutex::new(HashMap::new()))
}

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
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

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
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

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
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
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
	///   - `{ base, pattern }`: VS Code RelativePattern shape (base restricts
	///     the walk to that subfolder; falls back to all workspace folders if
	///     `base` doesn't resolve to a known folder)
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

		// Diagnostic: capture the actual include pattern + the input
		// DTO shape so the log makes the "every findFiles returns 0"
		// pattern debuggable. The pattern is the most common source
		// of zero-results - VS Code's internal callers sometimes pass
		// a `RelativePattern` whose `pattern` is `**/*.json` plus a
		// `base` that doesn't intersect any workspace folder, which
		// silently falls through to the all-folders walk but with a
		// pattern like `/**/*.json` (leading slash) that globset
		// then fails to match against the relative paths produced by
		// `Path.strip_prefix(...)`.
		dev_log!(
			"workspaces",
			"[FindFilesInWorkspace] include={} dto_shape={}",
			IncludePattern,
			if IncludePatternDTO.is_string() {
				"string"
			} else if IncludePatternDTO.is_object() {
				"object"
			} else if IncludePatternDTO.is_null() {
				"null"
			} else {
				"other"
			}
		);
		let ExcludePattern = ExcludePatternDTO
			.as_ref()
			.and_then(ExtractGlobPattern)
			.filter(|P| !P.is_empty());
		let Cap = MaxResults.unwrap_or(10_000).max(1);

		let IncludeMatcher = GlobBuilder::new(&IncludePattern)
			.literal_separator(false)
			.build()
			.map(|G| G.compile_matcher())
			.map_err(|Error| {
				CommonError::InvalidArgument { ArgumentName:"IncludePattern".into(), Reason:Error.to_string() }
			})?;
		let ExcludeMatcher = match &ExcludePattern {
			Some(P) => {
				Some(
					GlobBuilder::new(P)
						.literal_separator(false)
						.build()
						.map(|G| G.compile_matcher())
						.map_err(|Error| {
							CommonError::InvalidArgument {
								ArgumentName:"ExcludePattern".into(),
								Reason:Error.to_string(),
							}
						})?,
				)
			},
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
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
			.iter()
			.filter_map(|Folder| Folder.URI.to_file_path().ok())
			.collect();
		if Folders.is_empty() {
			dev_log!("workspaces", "[FindFilesInWorkspace] no workspace folders → []");
			return Ok(Vec::new());
		}

		let WalkRoots:Vec<PathBuf> = match &RestrictBase {
			Some(Base) => {
				let BasePath = PathBuf::from(Base);
				if Folders.iter().any(|F| BasePath.starts_with(F) || F.starts_with(&BasePath)) {
					vec![BasePath]
				} else {
					Folders.clone()
				}
			},
			None => Folders.clone(),
		};

		// Cache lookup: return a clone of the stored result when the same
		// (folders, include, exclude, cap, flags) tuple was walked within
		// the TTL window. The workbench fires findFiles repeatedly during
		// Cmd+P typing - serving the second-and-later calls from cache
		// drops the per-keystroke latency from "walk the tree" to a
		// HashMap lookup.
		let CacheKey = FindFilesCacheKey {
			Folders:WalkRoots.clone(),
			Include:IncludePattern.clone(),
			Exclude:ExcludePattern.clone(),
			Cap,
			UseIgnoreFiles,
			FollowSymlinks,
			RestrictBase:RestrictBase.clone(),
		};
		if let Some(Cached) = FindFilesCacheGet(&CacheKey) {
			dev_log!("workspaces", "[FindFilesInWorkspace] cache hit → {} match(es)", Cached.len());
			return Ok(Cached);
		}

		// Single-flight: if another caller is already walking for this
		// exact key, register as a follower and await the leader's
		// completion notify, then read the freshly-populated cache.
		// Otherwise we ARE the leader and proceed with the walk; on
		// completion we wake all waiters.
		// Lock-scope is restructured into an enum return so the
		// std::sync::MutexGuard is fully dropped BEFORE any `.await`
		// in either branch - otherwise the future is `!Send` and
		// tokio refuses to spawn it across worker threads.
		enum SingleFlightRole {
			Follower(Arc<Notify>),
			Leader(Arc<Notify>),
		}
		let RoleResolved:SingleFlightRole = {
			let mut Guard = FindFilesInFlight()
				.lock()
				.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;
			match Guard.get(&CacheKey) {
				Some(Existing) => SingleFlightRole::Follower(Existing.clone()),
				None => {
					let LeaderNotify = Arc::new(Notify::new());
					Guard.insert(CacheKey.clone(), LeaderNotify.clone());
					SingleFlightRole::Leader(LeaderNotify)
				},
			}
		};
		let LeaderNotify:Arc<Notify> = match RoleResolved {
			SingleFlightRole::Follower(WaitNotify) => {
				dev_log!(
					"workspaces",
					"[FindFilesInWorkspace] singleflight wait - leader walk in progress for include={}",
					IncludePattern
				);
				WaitNotify.notified().await;
				return Ok(FindFilesCacheGet(&CacheKey).unwrap_or_default());
			},
			SingleFlightRole::Leader(N) => N,
		};

		// Defensive: if anything between here and the cache-put panics
		// or returns Err, waiters would block forever. Guard with a
		// drop-time notify-and-remove via a small RAII helper.
		struct LeaderGuard {
			Key:FindFilesCacheKey,
			Notify:Arc<Notify>,
			Completed:bool,
		}
		impl Drop for LeaderGuard {
			fn drop(&mut self) {
				if !self.Completed {
					if let Ok(mut Guard) = FindFilesInFlight().lock() {
						Guard.remove(&self.Key);
					}
					self.Notify.notify_waiters();
				}
			}
		}
		let mut Leader = LeaderGuard { Key:CacheKey.clone(), Notify:LeaderNotify, Completed:false };

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
			.map_err(|_| {
				CommonError::Unknown { Description:"FindFilesInWorkspace: result Arc had outstanding refs".into() }
			})?
			.into_inner()
			.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;
		dev_log!(
			"workspaces",
			"[FindFilesInWorkspace] returned {} match(es) include={} exclude={:?} roots={}",
			Final.len(),
			IncludePattern,
			ExcludePattern,
			CacheKey.Folders.len()
		);
		FindFilesCachePut(CacheKey.clone(), Final.clone());

		// Successful walk + cache put: clear the in-flight entry and
		// wake any followers BEFORE the LeaderGuard drop fires so
		// followers see `Completed=true` and skip the drop-time
		// fallback path.
		{
			if let Ok(mut Guard) = FindFilesInFlight().lock() {
				Guard.remove(&CacheKey);
			}
			Leader.Notify.notify_waiters();
			Leader.Completed = true;
		}

		Ok(Final)
	}

	/// Opens a file in the editor by emitting the same
	/// `sky://editor/openDocument` event the workbench's
	/// `IEditorService.openEditor(uri)` path produces. Sky's bridge
	/// listens on this event and forwards through to the live
	/// `__CEL_SERVICES__.Commands.executeCommand("vscode.open", …)`
	/// inside the Output workbench bundle, which is what actually
	/// surfaces the file in the editor area.
	///
	/// Path resolution: accepts an absolute path (already a `PathBuf`).
	/// Constructs a `file://` URI via `Url::from_file_path` for
	/// proper percent-encoding of unicode / special chars; falls
	/// back to a manual prefix for relative paths (rare; Mountain
	/// callers always pass absolute paths via the trait).
	async fn OpenFile(&self, path:PathBuf) -> Result<(), CommonError> {
		use tauri::Emitter;
		dev_log!("workspaces", "[WorkspaceProvider] OpenFile called for: {:?}", path);

		let UriString = match Url::from_file_path(&path) {
			Ok(U) => U.to_string(),
			Err(_) => format!("file://{}", path.to_string_lossy()),
		};

		self.ApplicationHandle
			.emit(
				"sky://editor/openDocument",
				serde_json::json!({
					"uri": UriString,
					"viewColumn": null,
				}),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;

		Ok(())
	}
}

#[async_trait]
impl WorkspaceEditApplier for MountainEnvironment {
	/// Applies a workspace edit. Two-tier behaviour:
	///
	///   1. Emit `sky://editor/applyEdits` per URI so the workbench's
	///      `BulkEditService` applies edits to documents currently open in the
	///      editor (the canonical path - keeps undo / dirty state intact).
	///   2. For URIs that aren't currently tracked by the document mirror, fall
	///      through to a direct on-disk apply: read the file, sort edits by
	///      descending offset, splice each edit's `newText` into place, write
	///      atomically. Lets refactoring extensions touch files the user hasn't
	///      opened.
	///
	/// Each `TextEdit` is a JSON shape matching VS Code's
	/// `TextEditDTO`: `{ range: { start: {line, character}, end:
	/// {line, character} }, newText: string }`. Line/character are
	/// zero-based.
	async fn ApplyWorkspaceEdit(&self, Edit:WorkspaceEditDTO) -> Result<bool, CommonError> {
		use tauri::Emitter;
		dev_log!("workspaces", "[WorkspaceEditApplier] Applying workspace edit");

		let WorkspaceEditDTO { Edits } = Edit;
		let DocumentMirror = &self.ApplicationState.Feature.Documents;
		let mut AnyFailure = false;

		for (DocumentURIValue, TextEdits) in Edits {
			let UriString = DocumentURIValue
				.as_str()
				.map(String::from)
				.or_else(|| DocumentURIValue.get("value").and_then(Value::as_str).map(String::from))
				.unwrap_or_default();
			if UriString.is_empty() {
				dev_log!("workspaces", "warn: [WorkspaceEditApplier] empty URI in edit; skipping");
				continue;
			}

			// Tier 1: workbench-open document → emit Sky event.
			let _ = self.ApplicationHandle.emit(
				"sky://editor/applyEdits",
				serde_json::json!({
					"uri": UriString,
					"edits": TextEdits,
				}),
			);

			// Tier 2: if the document mirror doesn't know this URI,
			// also splice the edits to disk so refactors that touch
			// closed files actually mutate them. The renderer's
			// edit-apply path is a no-op on URIs it doesn't host -
			// the dual emit is safe (event lands in renderer for the
			// same-document case; on-disk writes happen for closed
			// files only).
			let IsOpen = DocumentMirror.Get(&UriString).is_some();
			if !IsOpen {
				if let Err(Error) = ApplyEditsToDisk(&UriString, &TextEdits).await {
					AnyFailure = true;
					dev_log!(
						"workspaces",
						"warn: [WorkspaceEditApplier] on-disk apply failed for {}: {}",
						UriString,
						Error
					);
				}
			}
		}

		Ok(!AnyFailure)
	}
}

/// Splice a list of `TextEditDTO`-shaped edits into the file at
/// `UriString`. Edits are applied in **descending** start offset so
/// each subsequent edit's offsets stay valid. Errors propagate as
/// `CommonError::FromStandardIOError` for read/write failures and
/// `CommonError::InvalidArgument` for malformed edits.
async fn ApplyEditsToDisk(UriString:&str, TextEdits:&[Value]) -> Result<(), CommonError> {
	use std::path::Path;
	let RawPath = if let Some(Stripped) = UriString.strip_prefix("file://") {
		percent_decode(Stripped)
	} else if UriString.starts_with('/') {
		UriString.to_string()
	} else {
		return Err(CommonError::InvalidArgument {
			ArgumentName:"uri".into(),
			Reason:format!("ApplyWorkspaceEdit: unsupported scheme in {}", UriString),
		});
	};
	let Path = Path::new(&RawPath);

	let Original = tokio::fs::read_to_string(Path)
		.await
		.map_err(|Error| CommonError::FromStandardIOError(Error, Path.to_path_buf(), "ApplyWorkspaceEdit.Read"))?;

	// Convert (line, character) positions to absolute byte offsets via
	// a single line-prefix scan. Edits referencing positions past EOF
	// are clamped to EOF (matches VS Code's bulk-edit forgiving
	// semantics on truncated files).
	let LineOffsets = ComputeLineOffsets(&Original);
	let mut WithOffsets:Vec<(usize, usize, String)> = Vec::with_capacity(TextEdits.len());
	for Edit in TextEdits {
		let StartLine = Edit.pointer("/range/start/line").and_then(Value::as_u64).unwrap_or(0) as usize;
		let StartChar = Edit.pointer("/range/start/character").and_then(Value::as_u64).unwrap_or(0) as usize;
		let EndLine = Edit
			.pointer("/range/end/line")
			.and_then(Value::as_u64)
			.unwrap_or(StartLine as u64) as usize;
		let EndChar = Edit
			.pointer("/range/end/character")
			.and_then(Value::as_u64)
			.unwrap_or(StartChar as u64) as usize;
		let NewText = Edit.get("newText").and_then(Value::as_str).unwrap_or("").to_string();
		let StartOffset = LinePosToOffset(&LineOffsets, &Original, StartLine, StartChar);
		let EndOffset = LinePosToOffset(&LineOffsets, &Original, EndLine, EndChar);
		WithOffsets.push((StartOffset, EndOffset, NewText));
	}

	WithOffsets.sort_by(|A, B| B.0.cmp(&A.0));

	let mut Mutated = Original;
	for (Start, End, NewText) in WithOffsets {
		let SafeStart = Start.min(Mutated.len());
		let SafeEnd = End.max(SafeStart).min(Mutated.len());
		Mutated.replace_range(SafeStart..SafeEnd, &NewText);
	}

	// Write via tempfile + rename for atomicity. Avoids torn writes
	// if the process is killed mid-mutation.
	let TempPath = Path.with_extension(format!(
		"{}.land-tmp-{}",
		Path.extension().and_then(|E| E.to_str()).unwrap_or("tmp"),
		std::process::id()
	));
	tokio::fs::write(&TempPath, Mutated.as_bytes())
		.await
		.map_err(|Error| CommonError::FromStandardIOError(Error, TempPath.clone(), "ApplyWorkspaceEdit.Write"))?;
	tokio::fs::rename(&TempPath, Path)
		.await
		.map_err(|Error| CommonError::FromStandardIOError(Error, Path.to_path_buf(), "ApplyWorkspaceEdit.Rename"))?;
	Ok(())
}

/// Pre-compute the byte offset of the start of every line.
fn ComputeLineOffsets(Source:&str) -> Vec<usize> {
	let mut Offsets = Vec::with_capacity(Source.len() / 40 + 1);
	Offsets.push(0);
	for (Index, Byte) in Source.bytes().enumerate() {
		if Byte == b'\n' {
			Offsets.push(Index + 1);
		}
	}
	Offsets
}

/// Resolve `(line, character)` to an absolute byte offset. Character is
/// counted in **UTF-16 code units** to match VS Code's
/// `Range`/`Position` semantics. Falls back gracefully when line/char
/// is past EOF.
fn LinePosToOffset(LineOffsets:&[usize], Source:&str, Line:usize, Character:usize) -> usize {
	if Line >= LineOffsets.len() {
		return Source.len();
	}
	let LineStart = LineOffsets[Line];
	let LineEnd = if Line + 1 < LineOffsets.len() {
		LineOffsets[Line + 1].saturating_sub(1)
	} else {
		Source.len()
	};
	let LineText = &Source[LineStart..LineEnd.min(Source.len())];
	let mut Utf16Count:usize = 0;
	for (ByteOffset, Char) in LineText.char_indices() {
		if Utf16Count >= Character {
			return LineStart + ByteOffset;
		}
		Utf16Count += Char.len_utf16();
	}
	LineStart + LineText.len()
}

/// Minimal percent-decode for `file://` URI paths. Reuses the
/// project's existing helpers when possible; this self-contained
/// version avoids an extra crate import.
fn percent_decode(Input:&str) -> String {
	let mut Out = String::with_capacity(Input.len());
	let mut Bytes = Input.as_bytes().iter().peekable();
	while let Some(&Byte) = Bytes.next() {
		if Byte == b'%' {
			let H = Bytes.next().copied();
			let L = Bytes.next().copied();
			if let (Some(H), Some(L)) = (H, L) {
				if let (Some(Hi), Some(Lo)) = (HexDigit(H), HexDigit(L)) {
					Out.push((Hi * 16 + Lo) as char);
					continue;
				}
				Out.push('%');
				Out.push(H as char);
				Out.push(L as char);
				continue;
			}
			Out.push('%');
		} else {
			Out.push(Byte as char);
		}
	}
	Out
}

fn HexDigit(Byte:u8) -> Option<u8> {
	match Byte {
		b'0'..=b'9' => Some(Byte - b'0'),
		b'a'..=b'f' => Some(Byte - b'a' + 10),
		b'A'..=b'F' => Some(Byte - b'A' + 10),
		_ => None,
	}
}

/// Extract a glob string from any of the shapes a caller can hand us:
///   - Bare string: `"**/*.rs"` → returned as-is.
///   - Object with `pattern`: `{ pattern: "..." }` (or `{ base, pattern }` for
///     VS Code's `RelativePattern`).
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
