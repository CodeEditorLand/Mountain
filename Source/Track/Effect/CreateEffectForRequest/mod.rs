/// Utilities module.
pub mod Utilities;

/// Authentication module.
pub mod Authentication;

/// Clipboard module.
pub mod Clipboard;

/// Commands module.
pub mod Commands;

/// Configuration module.
pub mod Configuration;

/// Debug module.
pub mod Debug;

/// Diagnostics module.
pub mod Diagnostics;

/// Documents module.
pub mod Documents;

/// Filesystem module.
pub mod FileSystem;

/// Filewatcher module.
pub mod FileWatcher;

/// Git module.
pub mod Git;

/// Keybinding module.
pub mod Keybinding;

/// Languagefeatures module.
pub mod LanguageFeatures;

/// Languages module.
pub mod Languages;

/// Nativehost module.
pub mod NativeHost;

/// Scm module.
pub mod SCM;

/// Search module.
pub mod Search;

/// Secrets module.
pub mod Secrets;

/// Statusbar module.
pub mod StatusBar;

/// Storage module.
pub mod Storage;

/// Task module.
pub mod Task;

/// Terminal module.
pub mod Terminal;

/// Treeview module.
pub mod TreeView;

/// Userinterface module.
pub mod UserInterface;

/// Webview module.
pub mod Webview;

/// Windowui module.
pub mod WindowUI;

/// Workspace module.
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
