//! Dispatcher for the generic `process_mountain_request` gRPC endpoint.
//!
//! Legacy JSON-over-gRPC rail used by Cocoon's
//! `MountainGRPCClient.sendRequest(method, params)` for method names that
//! predate the typed proto endpoints.
//!
//! Dispatch shape: method names resolve to a `Route` discriminant through
//! one `Lazy<HashMap>` lookup (covering every alias), then a single
//! `match` on the copy-cheap enum invokes the atom handler - no
//! sequential string comparisons on the hot rail.

use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde_json::json;
use tonic::{Request, Response, Status};
use url::Url;
use CommonLibrary::{
	LanguageFeature::{
		DTO::PositionDTO::PositionDTO,
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
	UserInterface::DTO::MessageSeverity::MessageSeverity,
};
use ::Vine::Generated::{GenericRequest as GenericRequestMsg, GenericResponse};

use super::{Commands, FileSystem, Secrets, WindowDialogs};
use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

/// Dispatch target for one generic-request method name (aliases share a
/// variant).
#[derive(Clone, Copy)]
enum Route {
	FsReadFile,

	FsWriteFile,

	FsStat,

	FsReaddir,

	FsCreateDir,

	FsDelete,

	FsRename,

	CommandsExecute,

	ExecuteCommand,

	UnregisterCommand,

	ShowOpenDialog,

	ShowSaveDialog,

	ShowInputBox,

	OpenExternal,

	ShowTextDocument,

	ShowInformation,

	ShowWarning,

	ShowError,

	CreateStatusBarItem,

	SetStatusBarText,

	CreateWebviewPanel,

	SetWebviewHtml,

	FindFiles,

	FindTextInFiles,

	OpenDocument,

	SaveAll,

	ApplyEdit,

	GetSecret,

	StoreSecret,

	DeleteSecret,

	ReadFileUri,

	WriteFileUri,

	StatUri,

	ReaddirUri,

	CallHierarchy,

	TypeHierarchy,
}

static ROUTES:Lazy<HashMap<&'static str, Route>> = Lazy::new(|| {
	HashMap::from([
		// ---- File System ---- (Cocoon FileSystemService uses these paths)
		("fs.readFile", Route::FsReadFile),
		("file:read", Route::FsReadFile),
		("fs.writeFile", Route::FsWriteFile),
		("file:write", Route::FsWriteFile),
		("fs.stat", Route::FsStat),
		("file:stat", Route::FsStat),
		("fs.listDir", Route::FsReaddir),
		("fs.readdir", Route::FsReaddir),
		("file:readdir", Route::FsReaddir),
		("fs.createDir", Route::FsCreateDir),
		("file:mkdir", Route::FsCreateDir),
		("fs.delete", Route::FsDelete),
		("file:delete", Route::FsDelete),
		("fs.rename", Route::FsRename),
		("file:move", Route::FsRename),
		// ---- Commands ----
		("commands.execute", Route::CommandsExecute),
		("executeCommand", Route::ExecuteCommand),
		("unregisterCommand", Route::UnregisterCommand),
		// ---- Window dialogs (Window.ts method names) ----
		("UserInterface.ShowOpenDialog", Route::ShowOpenDialog),
		("UserInterface.ShowSaveDialog", Route::ShowSaveDialog),
		("UserInterface.ShowInputBox", Route::ShowInputBox),
		// ---- Native shell operations ----
		("openExternal", Route::OpenExternal),
		// ---- Window (Cocoon MountainGRPCClient format) ----
		("showTextDocument", Route::ShowTextDocument),
		("showInformation", Route::ShowInformation),
		("showWarning", Route::ShowWarning),
		("showError", Route::ShowError),
		("createStatusBarItem", Route::CreateStatusBarItem),
		("setStatusBarText", Route::SetStatusBarText),
		("createWebviewPanel", Route::CreateWebviewPanel),
		("setWebviewHtml", Route::SetWebviewHtml),
		// ---- Workspace (Cocoon MountainGRPCClient format) ----
		("findFiles", Route::FindFiles),
		("findTextInFiles", Route::FindTextInFiles),
		("openDocument", Route::OpenDocument),
		("saveAll", Route::SaveAll),
		("applyEdit", Route::ApplyEdit),
		// ---- Secret Storage (Cocoon MountainGRPCClient format) ----
		("getSecret", Route::GetSecret),
		("storeSecret", Route::StoreSecret),
		("deleteSecret", Route::DeleteSecret),
		// ---- FS aliases (Cocoon MountainGRPCClient uses different key names) ----
		("readFile", Route::ReadFileUri),
		("writeFile", Route::WriteFileUri),
		("stat", Route::StatUri),
		("readdir", Route::ReaddirUri),
		// ---- Call Hierarchy / Type Hierarchy (T1.5 Approach A) ----
		("$provideCallHierarchyItems", Route::CallHierarchy),
		("prepareCallHierarchy", Route::CallHierarchy),
		("$provideTypeHierarchyItems", Route::TypeHierarchy),
		("prepareTypeHierarchy", Route::TypeHierarchy),
	])
});

pub async fn Fn(
	Service:&CocoonServiceImpl,

	request:Request<GenericRequestMsg>,
) -> Result<Response<GenericResponse>, Status> {
	let Req = request.into_inner();

	let RequestId = Req.request_identifier;

	dev_log!(
		"cocoon",
		"[CocoonService] generic request: method={} id={}",
		Req.method,
		RequestId
	);

	// Deserialise the generic parameter bytes as a JSON value
	let Params:serde_json::Value = match Req.parameter.is_empty() {
		true => serde_json::Value::Null,

		false => serde_json::from_slice(&Req.parameter).unwrap_or(serde_json::Value::Null),
	};

	let Route = match ROUTES.get(Req.method.as_str()) {
		Some(Route) => *Route,

		None => {
			dev_log!("cocoon", "warn: [CocoonService] Unknown generic method: {}", Req.method);

			return Ok(FileSystem::ErrResponse(
				RequestId,
				-32601,
				format!("Method '{}' not found", Req.method),
			));
		},
	};

	let Env = &Service.environment;

	let Reply = match Route {
		Route::FsReadFile => FileSystem::ReadFile::Fn(RequestId, Params).await,

		Route::FsWriteFile => FileSystem::WriteFile::Fn(RequestId, Params).await,

		Route::FsStat => FileSystem::Stat::Fn(RequestId, Params).await,

		Route::FsReaddir => FileSystem::Readdir::Fn(RequestId, Params).await,

		Route::FsCreateDir => FileSystem::CreateDir::Fn(RequestId, Params).await,

		Route::FsDelete => FileSystem::Delete::Fn(RequestId, Params).await,

		Route::FsRename => FileSystem::Rename::Fn(RequestId, Params).await,

		Route::CommandsExecute => Commands::Execute::Fn(RequestId, Params, Env).await,

		Route::ExecuteCommand => Commands::ExecuteCommand::Fn(RequestId, Params, Env).await,

		Route::UnregisterCommand => Commands::UnregisterCommand::Fn(RequestId, Params, Env).await,

		Route::ShowOpenDialog => WindowDialogs::ShowOpenDialog::Fn(RequestId, Params, Env).await,

		Route::ShowSaveDialog => WindowDialogs::ShowSaveDialog::Fn(RequestId, Params, Env).await,

		Route::ShowInputBox => WindowDialogs::ShowInputBox::Fn(RequestId, Params, Env).await,

		Route::OpenExternal => WindowDialogs::OpenExternal::Fn(RequestId, Params, Env),

		Route::ShowTextDocument => WindowDialogs::ShowTextDocument::Fn(RequestId, Params, Env),

		Route::ShowInformation => WindowDialogs::ShowMessage::Fn(RequestId, Params, Env, MessageSeverity::Info).await,

		Route::ShowWarning => WindowDialogs::ShowMessage::Fn(RequestId, Params, Env, MessageSeverity::Warning).await,

		Route::ShowError => WindowDialogs::ShowMessage::Fn(RequestId, Params, Env, MessageSeverity::Error).await,

		Route::CreateStatusBarItem => WindowDialogs::CreateStatusBarItem::Fn(RequestId, Params, Env),

		Route::SetStatusBarText => WindowDialogs::SetStatusBarText::Fn(RequestId, Params, Env),

		Route::CreateWebviewPanel => WindowDialogs::CreateWebviewPanel::Fn(RequestId, Params, Env),

		Route::SetWebviewHtml => WindowDialogs::SetWebviewHtml::Fn(RequestId, Params, Env),

		Route::OpenDocument => WindowDialogs::OpenDocument::Fn(RequestId, Params, Env),

		Route::SaveAll => WindowDialogs::SaveAll::Fn(RequestId, Params, Env),

		Route::ApplyEdit => WindowDialogs::ApplyEdit::Fn(RequestId, Params, Env),

		Route::GetSecret => Secrets::Get::Fn(RequestId, Params, Env).await,

		Route::StoreSecret => Secrets::Store::Fn(RequestId, Params, Env).await,

		Route::DeleteSecret => Secrets::Delete::Fn(RequestId, Params, Env).await,

		Route::ReadFileUri => FileSystem::ReadFileUri::Fn(RequestId, Params).await,

		Route::WriteFileUri => FileSystem::WriteFileUri::Fn(RequestId, Params).await,

		Route::StatUri => FileSystem::StatUri::Fn(RequestId, Params).await,

		Route::ReaddirUri => FileSystem::ReaddirUri::Fn(RequestId, Params).await,

		// `findFiles` / `findTextInFiles` are called by Cocoon's
		// `workspace.findFiles()` / `workspace.findTextInFiles()`
		// API shims. Delegate to the real trait implementations
		// (`WorkspaceProvider::FindFilesInWorkspace`,
		// `SearchProvider::TextSearch`) which use `ignore::WalkBuilder`
		// + `grep-searcher` - respecting `.gitignore`, doing parallel
		// walks, and producing properly-constructed `Url` results.
		Route::FindFiles => {
			use CommonLibrary::Workspace::WorkspaceProvider::WorkspaceProvider;

			let Include = Params
				.get("pattern")
				.cloned()
				.or_else(|| Params.get("include").cloned())
				.unwrap_or(serde_json::Value::String("**".into()));

			let Exclude = Params.get("exclude").cloned().filter(|V| !V.is_null());

			let MaxResults = Params.get("maxResults").and_then(|V| V.as_u64()).map(|N| N as usize);

			let UseIgnoreFiles = Params.get("useIgnoreFiles").and_then(|V| V.as_bool()).unwrap_or(true);

			let FollowSymlinks = Params.get("followSymlinks").and_then(|V| V.as_bool()).unwrap_or(false);

			match Env
				.FindFilesInWorkspace(Include, Exclude, MaxResults, UseIgnoreFiles, FollowSymlinks)
				.await
			{
				Ok(Urls) => {
					FileSystem::OkResponse(
						RequestId,
						&json!({ "uris": Urls.into_iter().map(|U| U.to_string()).collect::<Vec<_>>() }),
					)
				},

				Err(Error) => FileSystem::ErrResponse(RequestId, -32000, format!("findFiles: {}", Error)),
			}
		},

		Route::FindTextInFiles => {
			use CommonLibrary::Search::SearchProvider::SearchProvider;

			// VS Code's `workspace.findTextInFiles` takes a
			// `TextSearchQuery` in field `pattern` (or passed flat
			// at the top level). Accept both shapes.
			let QueryValue = match Params.get("pattern") {
				Some(V) if V.is_object() => V.clone(),

				Some(V) if V.is_string() => {
					json!({
						"pattern": V.as_str().unwrap_or(""),
						"isRegExp": Params.get("isRegExp").and_then(|V| V.as_bool()).unwrap_or(false),
						"isCaseSensitive": Params.get("isCaseSensitive").and_then(|V| V.as_bool()).unwrap_or(false),
						"isWordMatch": Params.get("isWordMatch").and_then(|V| V.as_bool()).unwrap_or(false),
					})
				},

				_ => Params.clone(),
			};

			let OptionsValue = Params.get("options").cloned().unwrap_or(serde_json::Value::Null);

			match Env.TextSearch(QueryValue, OptionsValue).await {
				Ok(Matches) => FileSystem::OkResponse(RequestId, &json!({ "matches": Matches })),

				Err(Error) => FileSystem::ErrResponse(RequestId, -32000, format!("findTextInFiles: {}", Error)),
			}
		},

		// These method names come from Cocoon's language provider when the
		// user triggers F12 / Go to References / Call Hierarchy. They use
		// the generic JSON request channel instead of typed proto methods
		// because `PrepareCallHierarchy` was never added to Vine.proto.
		// Params shape: `{ uri, position: { line, character } }`.
		Route::CallHierarchy => {
			let URI_Raw = Params.get("uri").and_then(|V| V.as_str()).unwrap_or("");

			let Line = Params
				.get("position")
				.and_then(|P| P.get("line"))
				.and_then(|V| V.as_u64())
				.unwrap_or(0);

			let Char = Params
				.get("position")
				.and_then(|P| P.get("character"))
				.and_then(|V| V.as_u64())
				.unwrap_or(0);

			match Url::parse(URI_Raw) {
				Ok(DocURI) => {
					let Pos = PositionDTO { LineNumber:Line as u32, Column:Char as u32 };

					match Env.PrepareCallHierarchy(DocURI, Pos).await {
						Ok(Result) => FileSystem::OkResponse(RequestId, &Result),

						Err(Error) => {
							FileSystem::ErrResponse(RequestId, -32000, format!("prepareCallHierarchy: {}", Error))
						},
					}
				},

				Err(_) => FileSystem::OkResponse(RequestId, &serde_json::Value::Array(Vec::new())),
			}
		},

		Route::TypeHierarchy => {
			let URI_Raw = Params.get("uri").and_then(|V| V.as_str()).unwrap_or("");

			let Line = Params
				.get("position")
				.and_then(|P| P.get("line"))
				.and_then(|V| V.as_u64())
				.unwrap_or(0);

			let Char = Params
				.get("position")
				.and_then(|P| P.get("character"))
				.and_then(|V| V.as_u64())
				.unwrap_or(0);

			match Url::parse(URI_Raw) {
				Ok(DocURI) => {
					let Pos = PositionDTO { LineNumber:Line as u32, Column:Char as u32 };

					match Env.PrepareTypeHierarchy(DocURI, Pos).await {
						Ok(Result) => FileSystem::OkResponse(RequestId, &Result),

						Err(Error) => {
							FileSystem::ErrResponse(RequestId, -32000, format!("prepareTypeHierarchy: {}", Error))
						},
					}
				},

				Err(_) => FileSystem::OkResponse(RequestId, &serde_json::Value::Array(Vec::new())),
			}
		},
	};

	Ok(Reply)
}
