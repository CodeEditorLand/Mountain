//! # CreateEffectForRequest (Track)
//!
//! Central routing table that maps string-based commands/RPC methods to typed
//! effects ([`MappedEffect`]). Each domain module handles its own match arms
//! and returns `None` for unrecognised methods, enabling the chain-of-
//! responsibility pattern below.
//!
//! ## Domain modules
//!
//! | Module            | Commands handled                                         |
//! |-------------------|----------------------------------------------------------|
//! | Authentication    | `Authentication.GetSession`, `Authentication.GetAccounts`|
//! | Clipboard         | `Clipboard.Read`, `Clipboard.Write`                      |
//! | Commands          | `executeCommand`, `Command.Execute`, `Command.GetAll`    |
//! | Configuration     | `config.get`, `config.update`, `Configuration.*`        |
//! | Debug             | `Debug.Start`, `Debug.RegisterConfigurationProvider`, `Debug.Stop` |
//! | Diagnostics       | `Diagnostic.Set`, `Diagnostic.Clear`                    |
//! | Documents         | `Document.Save`, `Document.SaveAs`                      |
//! | FileSystem        | `FileSystem.*`, `FileWatcher.*`, `openDocument` aliases  |
//! | Git               | `$gitExec`                                               |
//! | Keybinding        | `Keybinding.GetResolved`                                 |
//! | LanguageFeatures  | `register_*_provider` (one arm per provider type)        |
//! | Languages         | `Languages.GetAll`                                       |
//! | NativeHost        | `NativeHost.OpenExternal`                                |
//! | SCM               | `$scm:*`, `vscode.diff`, `$scm:openDiff`                 |
//! | Search            | `findFiles`, `findTextInFiles`, `Search.TextSearch`      |
//! | Secrets           | `secrets.get`, `secrets.store`, `secrets.delete`        |
//! | StatusBar         | `$statusBar:*`, `$setStatusBarMessage`, `$disposeStatusBarMessage` |
//! | Storage           | `Storage.Get`, `Storage.Set`                             |
//! | Task              | `Task.Fetch`, `Task.Execute`                             |
//! | Terminal          | `$terminal:*`, `Terminal.*`                              |
//! | TreeView          | `$tree:register`, `tree.*`                               |
//! | UserInterface     | `UserInterface.*`, `Window.*`                            |
//! | Webview           | `$webview:*`, `webview.*`, `$resolveCustomEditor`        |
//! | Workspace         | `applyEdit`, `showTextDocument`, `$updateWorkspaceFolders` |

pub mod Utilities;

pub mod Authentication;

pub mod Clipboard;

pub mod Commands;

pub mod Configuration;

pub mod Debug;

pub mod Diagnostics;

pub mod Documents;

pub mod FileReadAlias;

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
			if let Some(Result) = $Module::CreateEffect::<R>(MethodName, Parameters.clone()) {
				return Result;
			}
		};
	}

	Try!(Commands);

	Try!(Configuration);

	Try!(Diagnostics);

	Try!(Documents);

	Try!(FileReadAlias);

	Try!(FileSystem);

	Try!(FileWatcher);

	Try!(Keybinding);

	Try!(LanguageFeatures);

	Try!(Languages);

	Try!(Search);

	Try!(Storage);

	Try!(StatusBar);

	Try!(Terminal);

	Try!(TreeView);

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
