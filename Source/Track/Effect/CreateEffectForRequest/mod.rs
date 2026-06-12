//! Domain-specific effect constructors, one file per domain. The `Fn` in
//! this module delegates to domain modules in priority order.
//!
//! ## Domain Sub-modules
//!
//! - `Authentication`, `Clipboard`, `Commands`, `Configuration`, `Debug`,
//!   `Diagnostics`, `Documents`, `FileSystem`, `FileWatcher`, `Git`,
//!   `Keybinding`, `LanguageFeatures`, `Languages`, `NativeHost`, `SCM`,
//!   `Search`, `Secrets`, `StatusBar`, `Storage`, `Task`, `Terminal`,
//!   `TreeView`, `UserInterface`, `Utilities`, `Webview`, `WindowUI`,
//!   `Workspace`

/// Shared utilities (param parsing, proxy helpers).
pub mod Utilities;

/// Shim interception layer — gets first crack at every gRPC method.
pub mod Shim;

/// Authentication effect constructors.
pub mod Authentication;

/// Clipboard effect constructors.
pub mod Clipboard;

/// Commands effect constructors.
pub mod Commands;

/// Configuration effect constructors.
pub mod Configuration;

/// Debug effect constructors.
pub mod Debug;

/// Diagnostics effect constructors.
pub mod Diagnostics;

/// Documents effect constructors.
pub mod Documents;

/// File system effect constructors.
pub mod FileSystem;

/// File watcher effect constructors.
pub mod FileWatcher;

/// Git effect constructors.
pub mod Git;

/// Keybinding effect constructors.
pub mod Keybinding;

/// Language features effect constructors.
pub mod LanguageFeatures;

/// Languages effect constructors.
pub mod Languages;

/// Native host effect constructors.
pub mod NativeHost;

/// Source control management effect constructors.
pub mod SCM;

/// Search effect constructors.
pub mod Search;

/// Secrets effect constructors.
pub mod Secrets;

/// Status bar effect constructors.
pub mod StatusBar;

/// Storage effect constructors.
pub mod Storage;

/// Task effect constructors.
pub mod Task;

/// Terminal effect constructors.
pub mod Terminal;

/// Tree view effect constructors.
pub mod TreeView;

/// User interface effect constructors.
pub mod UserInterface;

/// Webview effect constructors.
pub mod Webview;

/// Window UI effect constructors.
pub mod WindowUI;

/// Workspace effect constructors.
pub mod Workspace;

use serde_json::Value;
use tauri::{AppHandle, Runtime};

use crate::Track::Effect::MappedEffectType::MappedEffect;

/// Maps a string-based method name (command or RPC) to its corresponding effect
/// constructor, returning a boxed closure ([`MappedEffect`]) that can be
/// executed by the ApplicationRunTime.
/// Delegates to domain modules in priority order. The first module that returns
/// `Some(result)` wins; unknown methods fall through to an error.
pub fn Fn<R:Runtime>(
	_ApplicationHandle:&AppHandle<R>,

	MethodName:&str,

	Parameters:Value,
) -> Result<MappedEffect, String> {
	macro_rules! Try {
		($Module:ident) => {
			if $Module::Matches(MethodName) {
				if let Some(Result) = $Module::CreateEffect::<R>(MethodName, Parameters.clone()) {
					return Result;
				}

				return Err(format!(
					"{}: {} matched method but did not return a handler",
					MethodName,
					stringify!($Module)
				));
			}
		};
	}

	Try!(Shim);

	Try!(FileSystem);

	Try!(Configuration);

	Try!(TreeView);

	Try!(Commands);

	Try!(Terminal);

	Try!(Diagnostics);

	Try!(Documents);

	// NOTE: FileReadAlias folded into FileSystem; keep this line only if the alias
	// module is restored and updated to call into FileSystem for
	// `openDocument`/`readFile`/`stat`.

	Try!(FileWatcher);

	Try!(Keybinding);

	Try!(LanguageFeatures);

	Try!(Languages);

	Try!(Search);

	Try!(Storage);

	Try!(StatusBar);

	Try!(UserInterface);

	Try!(WindowUI);

	Try!(Webview);

	Try!(Debug);

	Try!(SCM);

	Try!(Workspace);

	Try!(Secrets);

	Try!(Clipboard);

	Try!(NativeHost);

	Try!(Git);

	Try!(Task);

	Try!(Authentication);

	crate::dev_log!("ipc", "warn: [EffectCreation] Unknown method: {}", MethodName);

	Err(format!("Unknown method: {}", MethodName))
}
