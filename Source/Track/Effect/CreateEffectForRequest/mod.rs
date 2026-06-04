pub mod Utilities;

pub mod Authentication;

pub mod Clipboard;

pub mod Commands;

pub mod Configuration;

pub mod Debug;

pub mod Diagnostics;

pub mod Documents;

pub mod FileSystem;

pub mod FileWatcher;

pub mod Git;

pub mod Keybinding;

pub mod LanguageFeatures;

pub mod Languages;

pub mod NativeHost;

pub mod SCM;

pub mod Search;

pub mod Secrets;

pub mod StatusBar;

pub mod Storage;

pub mod Task;

pub mod Terminal;

pub mod TreeView;

pub mod UserInterface;

pub mod Webview;

pub mod WindowUI;

pub mod Workspace;

use serde_json::Value;
use tauri::{AppHandle, Runtime};

use crate::Track::Effect::MappedEffectType::MappedEffect;

/// Maps a string-based method name (command or RPC) to its corresponding effect
/// constructor, returning a boxed closure ([`MappedEffect`]) that can be
/// executed by the ApplicationRunTime.
///
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
				if let Some(Result) = $Module::CreateEffect::<R>(MethodName, Parameters.clone()) { return Result; }
				return Err(format!("{}: {} matched method but did not return a handler", MethodName, stringify!($Module)));
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
	// module is restored and updated to call into FileSystem for `openDocument`/`readFile`/`stat`.

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
