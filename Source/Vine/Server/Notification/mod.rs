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
//! - Wire string `unregister_scm_provider` → atom file `UnregisterScmProvider.rs`.
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
//! - `Service` gives access to `ApplicationHandle` (for Tauri
//!   `emit` / webview lookup) and `RunTime` (for `Environment`,
//!   `ApplicationState`, provider registry, scheduler).
//! - `Parameter` is the raw JSON payload Cocoon sent; each atom extracts
//!   the fields it needs and validates locally.
//! - Return `()` - atoms that need to fail just log via `dev_log!` on
//!   the `notif-drop` / `grpc` tag; the caller always returns `Empty` to
//!   Cocoon because notifications are fire-and-forget.

#![allow(non_snake_case)]

// --- Batch 8: provider-unregister cleanup ---
pub mod UnregisterAuthenticationProvider;
pub mod UnregisterDebugAdapter;
pub mod UnregisterFileSystemProvider;
pub mod UnregisterScmProvider;
pub mod UnregisterTaskProvider;
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
