//! # Vine Cocoon → Mountain Notification Atoms
//!
//! One handler per file, file name = the exported function name
//! (reverse-hierarchical path: `Vine::Server::Notification::<Atom>::<Atom>`).
//! Each atom encapsulates exactly one wire-method's side effects so the
//! main `send_cocoon_notification` dispatcher in
//! `MountainVinegRPCService.rs` stays a thin match that routes into
//! these files.
//!
//! ## Naming
//!
//! - Wire string `outputChannel.create` → atom file `OutputChannelCreate.rs`
//!   with `pub async fn OutputChannelCreate(...)`.
//! - Wire string `unregister_scm_provider` → atom file
//!   `UnregisterScmProvider.rs`.
//! - Wire string `progress.update` → atom file `ProgressUpdate.rs`.
//!
//! Snake_case / dotted wire strings collapse to PascalCase file names.
//! The function name mirrors the file name verbatim so a grep for
//! `fn <Name>` lands in exactly one place.
//!
//! ## Signature contract
//!
//! Every atom takes the same two parameters:
//!
//! ```ignore
//! pub async fn <Atom>(
//!     Service: &MountainVinegRPCService,
//!     Parameter: &serde_json::Value,
//! );
//! ```
//!
//! - `Service` gives access to `ApplicationHandle` (for Tauri `emit` / webview
//!   lookup) and `RunTime` (for `Environment`, `ApplicationState`, provider
//!   registry, scheduler).
//! - `Parameter` is the raw JSON payload Cocoon sent; each atom extracts the
//!   fields it needs and validates locally.
//! - Return `()` - atoms that need to fail just log via `dev_log!` on the
//!   `notif-drop` / `grpc` tag; the caller always returns `Empty` to Cocoon
//!   because notifications are fire-and-forget.

// --- Shared support utilities ---
pub mod Support;

// --- Batch 8: provider-unregister cleanup ---
pub mod UnregisterAuthenticationProvider;

pub mod UnregisterDebugAdapter;

pub mod UnregisterDebugConfigurationProvider;

pub mod UnregisterFileSystemProvider;

pub mod UnregisterScmProvider;

pub mod UnregisterTaskProvider;

pub mod UnregisterExternalUriOpener;

pub mod UnregisterRemoteAuthorityResolver;

pub mod UnregisterUriHandler;

pub mod UpdateScmGroup;

// --- Batch 11: progress lifecycle name alignment ---
pub mod ProgressComplete;

pub mod ProgressUpdate;

// --- Batch 10: status-bar text + disposal ---
pub mod DisposeStatusBarItem;

pub mod SetStatusBarText;

// --- Batch 9: output channel lifecycle (`output.*` + `outputChannel.*`) ---
pub mod OutputAppend;

pub mod OutputAppendLine;

pub mod OutputChannelAppend;

pub mod OutputChannelCoalesce;

pub mod OutputChannelClear;

pub mod OutputChannelCreate;

pub mod OutputChannelDispose;

pub mod OutputChannelHide;

pub mod OutputChannelShow;

pub mod OutputClear;

pub mod OutputCreate;

pub mod OutputDispose;

pub mod OutputReplace;

pub mod OutputShow;

// --- Batch 13: webview reverse messaging ---
pub mod WebviewDispose;

pub mod WebviewPostMessage;

// --- Batch 14: grammar, security, external ---
pub mod OpenExternal;

pub mod SecurityIncident;

pub mod SetLanguageConfiguration;

// --- Batch 15: inline arms atomised from `MountainVinegRPCService` dispatcher.
// These were previously ~300 lines of inline match-arm bodies; now each
// wire method is a one-fn file that the dispatcher delegates into.
pub mod ExtensionActivated;

pub mod ExtensionDeactivated;

pub mod ExtensionHostMessage;

pub mod LanguagesSetDocumentLanguage;

pub mod ProgressEnd;

pub mod ProgressReport;

pub mod ProgressStart;

pub mod WebviewReady;

pub mod WindowShowTextDocument;

pub mod WorkspaceApplyEdit;

// --- Batch 16: the remaining inline arms - command register/unregister,
// status-bar lifecycle / message, window show-message / create-terminal,
// decoration / debug / webview / terminal fan-outs. A handful are
// "group" atoms (`TerminalLifecycle` covers 4 wire methods that share a
// relay + provider-drive pattern) - kept together where the handling
// is truly identical and splitting would duplicate 5-line files.
pub mod DebugLifecycle;

pub mod DecorationTypeLifecycle;

pub mod RegisterCommand;

pub mod StatusBarLifecycle;

pub mod StatusBarMessage;

pub mod TerminalLifecycle;

pub mod UnregisterCommand;

pub mod WebviewLifecycle;

pub mod WindowCreateTerminal;

pub mod WindowShowMessage;

// --- Batch 17 (post-§14): SCM register pair pulled out of the
// language-providers OR-block in `MountainVinegRPCService.rs`. The
// catch-all fallthrough was registering SCM providers in the
// language-feature provider registry, which the SCM viewlet never
// reads. These atoms route through `SourceControlManagementProvider`
// + emit the `sky://scm/*` events the renderer actually subscribes to.
pub mod RegisterScmProvider;

pub mod RegisterScmResourceGroup;

// --- Batch 18: language-provider OR-block extracted from the inline match
// in `MountainVinegRPCService::send_cocoon_notification`. All 46+
// `register_*` / `register_*_provider` variants now delegate here via a
// single `RegisterLanguageProvider(service, method, params).await` call.
pub mod RegisterLanguageProvider;

// --- Text editor API ---
// Atoms for `editor.setDecorations(type, ranges)` and `editor.edit(cb)`.
// These complete the decoration pipeline and enable in-place text mutations
// from extensions (formatters, code actions, vim-mode, Error Lens, etc.).
pub mod SetTextEditorDecorations;

pub mod ApplyTextEdits;
