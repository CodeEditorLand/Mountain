//! Dispatcher for the generic `process_mountain_request` gRPC endpoint.
//!
//! Legacy JSON-over-gRPC rail used by Cocoon's
//! `MountainGRPCClient.sendRequest(method, params)` for method names that
//! predate the typed proto endpoints. Match arms call into Mountain's
//! environment directly via `Service.environment.*`.

use std::time::UNIX_EPOCH;

use serde_json::json;

use tonic::{Request, Response, Status};

use url::Url;

use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	LanguageFeature::{
		DTO::PositionDTO::PositionDTO,
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
};

use ::Vine::Generated::{GenericRequest as GenericRequestMsg, GenericResponse, RpcError};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

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

	/// Serialise a value into the `result` bytes of a GenericResponse.
	fn OkResponse(RequestId:u64, Value:&impl serde::Serialize) -> Response<GenericResponse> {
		let Bytes = serde_json::to_vec(Value).unwrap_or_default();

		Response::new(GenericResponse { request_identifier:RequestId, result:Bytes, error:None })
	}

	/// Build an error GenericResponse.
	fn ErrResponse(RequestId:u64, Code:i32, Message:String) -> Response<GenericResponse> {
		Response::new(GenericResponse {
			request_identifier:RequestId,
			result:Vec::new(),
			error:Some(RpcError { code:Code, message:Message, data:Vec::new() }),
		})
	}

	// Deserialise the generic parameter bytes as a JSON value
	let Params:serde_json::Value = if Req.parameter.is_empty() {
		serde_json::Value::Null
	} else {
		serde_json::from_slice(&Req.parameter).unwrap_or(serde_json::Value::Null)
	};

	match Req.method.as_str() {
		// ---- File System ---- (Cocoon FileSystemService uses these paths)
		"fs.readFile" | "file:read" => {
			let Path = Params
				.as_str()
				.or_else(|| Params.get("path").and_then(|V| V.as_str()))
				.unwrap_or("");

			match tokio::fs::read(Path).await {
				Ok(Content) => Ok(OkResponse(RequestId, &Content)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.readFile: {}", Error))),
			}
		},

		"fs.writeFile" | "file:write" => {
			let Path = Params.get("path").and_then(|V| V.as_str()).unwrap_or("");

			let Content:Vec<u8> = Params
				.get("content")
				.and_then(|V| V.as_array())
				.map(|A| A.iter().filter_map(|B| B.as_u64().map(|N| N as u8)).collect())
				.unwrap_or_default();

			match tokio::fs::write(Path, &Content).await {
				Ok(()) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.writeFile: {}", Error))),
			}
		},

		"fs.stat" | "file:stat" => {
			let Path = Params
				.as_str()
				.or_else(|| Params.get("path").and_then(|V| V.as_str()))
				.unwrap_or("");

			match tokio::fs::metadata(Path).await {
				Ok(Meta) => {
					let Mtime = Meta
						.modified()
						.ok()
						.and_then(|T| T.duration_since(UNIX_EPOCH).ok())
						.map(|D| D.as_millis() as u64)
						.unwrap_or(0);

					Ok(OkResponse(
						RequestId,

						&json!({
							"type": if Meta.is_dir() { 2 } else { 1 },
							"is_file": Meta.is_file(),
							"is_directory": Meta.is_dir(),
							"size": Meta.len(),
							"mtime": Mtime,
						}),
					))
				},

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.stat: {}", Error))),
			}
		},

		"fs.listDir" | "fs.readdir" | "file:readdir" => {
			let Path = Params
				.as_str()
				.or_else(|| Params.get("path").and_then(|V| V.as_str()))
				.unwrap_or("");

			match tokio::fs::read_dir(Path).await {
				Ok(mut Entries) => {
					// Return [{name, type}] where type 1=File 2=Directory
					let mut Items:Vec<serde_json::Value> = Vec::new();

					while let Ok(Some(Entry)) = Entries.next_entry().await {
						if let Some(Name) = Entry.file_name().to_str() {
							let IsDir = Entry.file_type().await.map(|T| T.is_dir()).unwrap_or(false);

							Items.push(json!({ "name": Name, "type": if IsDir { 2u32 } else { 1u32 } }));
						}
					}

					Ok(OkResponse(RequestId, &Items))
				},

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.listDir: {}", Error))),
			}
		},

		"fs.createDir" | "file:mkdir" => {
			let Path = Params
				.as_str()
				.or_else(|| Params.get("path").and_then(|V| V.as_str()))
				.unwrap_or("");

			match tokio::fs::create_dir_all(Path).await {
				Ok(()) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.createDir: {}", Error))),
			}
		},

		"fs.delete" | "file:delete" => {
			let Path = Params
				.as_str()
				.or_else(|| Params.get("path").and_then(|V| V.as_str()))
				.unwrap_or("");

			let Result = if std::path::Path::new(Path).is_dir() {
				tokio::fs::remove_dir_all(Path).await
			} else {
				tokio::fs::remove_file(Path).await
			};

			match Result {
				Ok(()) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.delete: {}", Error))),
			}
		},

		"fs.rename" | "file:move" => {
			let From = Params.get("from").and_then(|V| V.as_str()).unwrap_or("");

			let To = Params.get("to").and_then(|V| V.as_str()).unwrap_or("");

			match tokio::fs::rename(From, To).await {
				Ok(()) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("fs.rename: {}", Error))),
			}
		},

		// ---- Commands ----
		"commands.execute" => {
			let CommandId = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Arg = Params.get("arg").cloned().unwrap_or(serde_json::Value::Null);

			match Service.environment.ExecuteCommand(CommandId, Arg).await {
				Ok(Value) => Ok(OkResponse(RequestId, &Value)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		// ---- Commands (Cocoon MountainGRPCClient format) ----
		"executeCommand" => {
			let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Arg = Params
				.get("arguments")
				.and_then(|A| A.as_array())
				.and_then(|A| A.first())
				.cloned()
				.unwrap_or(serde_json::Value::Null);

			match Service.environment.ExecuteCommand(CommandId, Arg).await {
				Ok(Value) => Ok(OkResponse(RequestId, &json!({ "result": Value }))),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		"unregisterCommand" => {
			let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			match Service.environment.UnregisterCommand(ExtensionId, CommandId).await {
				Ok(()) => Ok(OkResponse(RequestId, &json!({ "success": true }))),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		// ---- Window dialogs (Window.ts method names) ----
		"UserInterface.ShowOpenDialog" => {
			use CommonLibrary::UserInterface::{
				DTO::OpenDialogOptionsDTO::OpenDialogOptionsDTO,
				UserInterfaceProvider::UserInterfaceProvider,
			};

			let Title = Params
				.get(0)
				.and_then(|V| V.get("title"))
				.and_then(|T| T.as_str())
				.map(|S| S.to_string());

			let Options = OpenDialogOptionsDTO {
				Base:CommonLibrary::UserInterface::DTO::DialogOptionsDTO::DialogOptionsDTO {
					Title,
					..Default::default()
				},
				..OpenDialogOptionsDTO::default()
			};

			match Service.environment.ShowOpenDialog(Some(Options)).await {
				Ok(Some(Paths)) => {
					let Uris:Vec<String> = Paths.iter().map(|P| format!("file://{}", P.display())).collect();

					Ok(OkResponse(RequestId, &json!(Uris)))
				},

				Ok(None) => Ok(OkResponse(RequestId, &json!(serde_json::Value::Array(vec![])))),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		"UserInterface.ShowSaveDialog" => {
			use CommonLibrary::UserInterface::{
				DTO::SaveDialogOptionsDTO::SaveDialogOptionsDTO,
				UserInterfaceProvider::UserInterfaceProvider,
			};

			let Title = Params
				.get(0)
				.and_then(|V| V.get("title"))
				.and_then(|T| T.as_str())
				.map(|S| S.to_string());

			let Options = SaveDialogOptionsDTO {
				Base:CommonLibrary::UserInterface::DTO::DialogOptionsDTO::DialogOptionsDTO {
					Title,
					..Default::default()
				},
				..SaveDialogOptionsDTO::default()
			};

			match Service.environment.ShowSaveDialog(Some(Options)).await {
				Ok(Some(Path)) => Ok(OkResponse(RequestId, &json!(format!("file://{}", Path.display())))),

				Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		"UserInterface.ShowInputBox" => {
			use CommonLibrary::UserInterface::{
				DTO::InputBoxOptionsDTO::InputBoxOptionsDTO,
				UserInterfaceProvider::UserInterfaceProvider,
			};

			let Opts = Params.get(0);

			let Options = InputBoxOptionsDTO {
				Prompt:Opts
					.and_then(|V| V.get("prompt"))
					.and_then(|P| P.as_str())
					.map(|S| S.to_string()),

				PlaceHolder:Opts
					.and_then(|V| V.get("placeHolder"))
					.and_then(|P| P.as_str())
					.map(|S| S.to_string()),

				IsPassword:Some(Opts.and_then(|V| V.get("password")).and_then(|B| B.as_bool()).unwrap_or(false)),

				Value:Opts
					.and_then(|V| V.get("value"))
					.and_then(|V| V.as_str())
					.map(|S| S.to_string()),

				Title:None,

				IgnoreFocusOut:None,
			};

			match Service.environment.ShowInputBox(Some(Options)).await {
				Ok(Some(Text)) => Ok(OkResponse(RequestId, &json!(Text))),

				Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		// ---- Native shell operations ----
		"openExternal" => {
			use tauri::Emitter;

			let Url = Params
				.as_str()
				.or_else(|| Params.get("url").and_then(|V| V.as_str()))
				.unwrap_or("")
				.to_string();

			// Emit to Sky - Sky uses Tauri shell plugin to open the URL
			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://native/openExternal", json!({ "url": Url }));

			Ok(OkResponse(RequestId, &json!({ "success": true })))
		},

		// ---- Window (Cocoon MountainGRPCClient format) ----
		"showTextDocument" => {
			use tauri::Emitter;

			let Uri = Params
				.get("uri")
				.and_then(|V| V.get("value").or(Some(V)))
				.and_then(|V| V.as_str())
				.unwrap_or("")
				.to_string();

			let ViewColumn = Params.get("viewColumn").and_then(|V| V.as_i64()).map(|N| N + 2);

			let PreserveFocus = Params.get("preserveFocus").and_then(|V| V.as_bool()).unwrap_or(false);

			let _ = Service.environment.ApplicationHandle.emit(
				"sky://editor/openDocument",

				json!({ "uri": Uri, "viewColumn": ViewColumn, "preserveFocus": PreserveFocus }),
			);

			Ok(OkResponse(RequestId, &json!({ "success": true })))
		},

		"showInformation" => {
			use CommonLibrary::UserInterface::{
				DTO::MessageSeverity::MessageSeverity,
				UserInterfaceProvider::UserInterfaceProvider,
			};

			let Message = Params.get("message").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Items:Option<serde_json::Value> = Params
				.get("items")
				.cloned()
				.filter(|V| V.is_array() && !V.as_array().unwrap().is_empty());

			match Service.environment.ShowMessage(MessageSeverity::Info, Message, Items).await {
				Ok(Some(Selected)) => Ok(OkResponse(RequestId, &json!({ "selectedItem": Selected }))),

				Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		"showWarning" => {
			use CommonLibrary::UserInterface::{
				DTO::MessageSeverity::MessageSeverity,
				UserInterfaceProvider::UserInterfaceProvider,
			};

			let Message = Params.get("message").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Items:Option<serde_json::Value> = Params
				.get("items")
				.cloned()
				.filter(|V| V.is_array() && !V.as_array().unwrap().is_empty());

			match Service.environment.ShowMessage(MessageSeverity::Warning, Message, Items).await {
				Ok(Some(Selected)) => Ok(OkResponse(RequestId, &json!({ "selectedItem": Selected }))),

				Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		"showError" => {
			use CommonLibrary::UserInterface::{
				DTO::MessageSeverity::MessageSeverity,
				UserInterfaceProvider::UserInterfaceProvider,
			};

			let Message = Params.get("message").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Items:Option<serde_json::Value> = Params
				.get("items")
				.cloned()
				.filter(|V| V.is_array() && !V.as_array().unwrap().is_empty());

			match Service.environment.ShowMessage(MessageSeverity::Error, Message, Items).await {
				Ok(Some(Selected)) => Ok(OkResponse(RequestId, &json!({ "selectedItem": Selected }))),

				Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		"createStatusBarItem" => {
			use tauri::Emitter;

			let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Tooltip = Params.get("tooltip").and_then(|V| V.as_str()).unwrap_or("").to_string();

			// Sky's `SetOrUpdateEntry` (`SkyBridge.ts:744`) listens on
			// `sky://statusbar/set-entry` and `sky://statusbar/update`
			// - both route through the same upsert. There is no
			// `sky://statusbar/create` listener; emit the canonical
			// `set-entry` channel so the entry materialises on first
			// register.
			let _ = Service.environment.ApplicationHandle.emit(
				"sky://statusbar/set-entry",

				json!({ "id": Id, "text": Text, "tooltip": Tooltip }),
			);

			Ok(OkResponse(RequestId, &json!({ "itemId": Id })))
		},

		"setStatusBarText" => {
			use tauri::Emitter;

			let ItemId = Params.get("itemId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://statusbar/update", json!({ "id": ItemId, "text": Text }));

			Ok(OkResponse(RequestId, &json!({ "success": true })))
		},

		"createWebviewPanel" => {
			use tauri::Emitter;

			let ViewType = Params.get("viewType").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Title = Params.get("title").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Handle = std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|D| D.as_millis() as u64)
				.unwrap_or(0);

			let _ = Service.environment.ApplicationHandle.emit("sky://webview/create", json!({ "handle": Handle, "viewType": ViewType, "title": Title, "viewColumn": Params.get("viewColumn"), "preserveFocus": Params.get("preserveFocus").and_then(|V| V.as_bool()).unwrap_or(false) }));

			Ok(OkResponse(RequestId, &json!({ "handle": Handle })))
		},

		"setWebviewHtml" => {
			use tauri::Emitter;

			let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0);

			let Html = Params.get("html").and_then(|V| V.as_str()).unwrap_or("").to_string();

			// Canonical kebab-case channel; `sky://webview/setHtml` retired.
			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://webview/set-html", json!({ "handle": Handle, "html": Html }));

			Ok(OkResponse(RequestId, &json!({ "success": true })))
		},

		// ---- Workspace (Cocoon MountainGRPCClient format) ----
		// `findFiles` / `findTextInFiles` are called by Cocoon's
		// `workspace.findFiles()` / `workspace.findTextInFiles()`
		// API shims. Delegate to the real trait implementations
		// (`WorkspaceProvider::FindFilesInWorkspace`,
		// `SearchProvider::TextSearch`) which use `ignore::WalkBuilder`
		// + `grep-searcher` - respecting `.gitignore`, doing parallel
		// walks, and producing properly-constructed `Url` results.
		// Prior inline implementations used naive dir-walks, hidden-
		// dot skipping, and `format!("file://{}", path)` URI
		// construction that mangled non-ASCII paths.
		"findFiles" => {
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

			match Service
				.environment
				.FindFilesInWorkspace(Include, Exclude, MaxResults, UseIgnoreFiles, FollowSymlinks)
				.await
			{
				Ok(Urls) => {
					Ok(OkResponse(
						RequestId,

						&json!({ "uris": Urls.into_iter().map(|U| U.to_string()).collect::<Vec<_>>() }),
					))
				},

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("findFiles: {}", Error))),
			}
		},

		"findTextInFiles" => {
			use CommonLibrary::Search::SearchProvider::SearchProvider;

			// VS Code's `workspace.findTextInFiles` takes a
			// `TextSearchQuery` in field `pattern` (or passed flat
			// at the top level). Accept both shapes.
			let QueryValue = if Params.get("pattern").map(|V| V.is_object()).unwrap_or(false) {
				Params.get("pattern").cloned().unwrap_or(serde_json::Value::Null)
			} else if Params.get("pattern").map(|V| V.is_string()).unwrap_or(false) {
				json!({
					"pattern": Params.get("pattern").and_then(|V| V.as_str()).unwrap_or(""),
					"isRegExp": Params.get("isRegExp").and_then(|V| V.as_bool()).unwrap_or(false),
					"isCaseSensitive": Params.get("isCaseSensitive").and_then(|V| V.as_bool()).unwrap_or(false),
					"isWordMatch": Params.get("isWordMatch").and_then(|V| V.as_bool()).unwrap_or(false),
				})
			} else {
				Params.clone()
			};

			let OptionsValue = Params.get("options").cloned().unwrap_or(serde_json::Value::Null);

			match Service.environment.TextSearch(QueryValue, OptionsValue).await {
				Ok(Matches) => Ok(OkResponse(RequestId, &json!({ "matches": Matches }))),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("findTextInFiles: {}", Error))),
			}
		},

		"openDocument" => {
			use tauri::Emitter;

			let Uri = Params
				.get("uri")
				.and_then(|V| V.get("value").or(Some(V)))
				.and_then(|V| V.as_str())
				.unwrap_or("")
				.to_string();

			let ViewColumn = Params.get("viewColumn").and_then(|V| V.as_i64());

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://editor/openDocument", json!({ "uri": Uri, "viewColumn": ViewColumn }));

			Ok(OkResponse(RequestId, &json!({ "success": true })))
		},

		"saveAll" => {
			use tauri::Emitter;

			let IncludeUntitled = Params.get("includeUntitled").and_then(|V| V.as_bool()).unwrap_or(false);

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://editor/saveAll", json!({ "includeUntitled": IncludeUntitled }));

			Ok(OkResponse(RequestId, &json!({ "success": true })))
		},

		"applyEdit" => {
			use tauri::Emitter;

			let Uri = Params
				.get("uri")
				.and_then(|V| V.get("value").or(Some(V)))
				.and_then(|V| V.as_str())
				.unwrap_or("")
				.to_string();

			let Edits = Params.get("edits").cloned().unwrap_or(json!([]));

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://editor/applyEdits", json!({ "uri": Uri, "edits": Edits }));

			Ok(OkResponse(RequestId, &json!({ "success": true })))
		},

		// ---- Secret Storage (Cocoon MountainGRPCClient format) ----
		"getSecret" => {
			use CommonLibrary::Secret::SecretProvider::SecretProvider;

			let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();

			match Service.environment.GetSecret(ExtensionId, Key).await {
				Ok(Some(Value)) => Ok(OkResponse(RequestId, &json!({ "value": Value }))),

				Ok(None) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		"storeSecret" => {
			use CommonLibrary::Secret::SecretProvider::SecretProvider;

			let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Value = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

			match Service.environment.StoreSecret(ExtensionId, Key, Value).await {
				Ok(()) => Ok(OkResponse(RequestId, &json!({ "success": true }))),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		"deleteSecret" => {
			use CommonLibrary::Secret::SecretProvider::SecretProvider;

			let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();

			match Service.environment.DeleteSecret(ExtensionId, Key).await {
				Ok(()) => Ok(OkResponse(RequestId, &json!({ "success": true }))),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, Error.to_string())),
			}
		},

		// ---- FS aliases (Cocoon MountainGRPCClient uses different key names) ----
		"readFile" => {
			let Uri = Params
				.get("uri")
				.and_then(|V| V.as_str())
				.or_else(|| Params.as_str())
				.unwrap_or("")
				.replace("file://", "");

			match tokio::fs::read(&Uri).await {
				Ok(Content) => Ok(OkResponse(RequestId, &Content)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("readFile: {}", Error))),
			}
		},

		"writeFile" => {
			let Uri = Params.get("uri").and_then(|V| V.as_str()).unwrap_or("").replace("file://", "");

			let Content:Vec<u8> = Params
				.get("content")
				.and_then(|V| V.as_array())
				.map(|A| A.iter().filter_map(|B| B.as_u64().map(|N| N as u8)).collect())
				.unwrap_or_default();

			match tokio::fs::write(&Uri, &Content).await {
				Ok(()) => Ok(OkResponse(RequestId, &serde_json::Value::Null)),

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("writeFile: {}", Error))),
			}
		},

		"stat" => {
			let Uri = Params
				.get("uri")
				.and_then(|V| V.as_str())
				.or_else(|| Params.as_str())
				.unwrap_or("")
				.replace("file://", "");

			match tokio::fs::metadata(&Uri).await {
				Ok(Meta) => {
					let Mtime = Meta
						.modified()
						.ok()
						.and_then(|T| T.duration_since(UNIX_EPOCH).ok())
						.map(|D| D.as_millis() as u64)
						.unwrap_or(0);

					Ok(OkResponse(
						RequestId,

						&json!({ "type": if Meta.is_dir() { 2 } else { 1 }, "is_file": Meta.is_file(), "is_directory": Meta.is_dir(), "size": Meta.len(), "mtime": Mtime }),
					))
				},

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("stat: {}", Error))),
			}
		},

		"readdir" => {
			let Uri = Params
				.get("uri")
				.and_then(|V| V.as_str())
				.or_else(|| Params.as_str())
				.unwrap_or("")
				.replace("file://", "");

			match tokio::fs::read_dir(&Uri).await {
				Ok(mut Entries) => {
					let mut Names:Vec<String> = Vec::new();

					while let Ok(Some(Entry)) = Entries.next_entry().await {
						if let Some(Name) = Entry.file_name().to_str() {
							Names.push(Name.to_string());
						}
					}

					Ok(OkResponse(RequestId, &Names))
				},

				Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("readdir: {}", Error))),
			}
		},

		// ---- Call Hierarchy / Type Hierarchy (T1.5 Approach A) ----
		// These method names come from Cocoon's language provider when the
		// user triggers F12 / Go to References / Call Hierarchy. They use
		// the generic JSON request channel instead of typed proto methods
		// because `PrepareCallHierarchy` was never added to Vine.proto.
		// Params shape: `{ uri, position: { line, character } }`.
		"$provideCallHierarchyItems" | "prepareCallHierarchy" => {
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

					match Service.environment.PrepareCallHierarchy(DocURI, Pos).await {
						Ok(Result) => Ok(OkResponse(RequestId, &Result)),

						Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("prepareCallHierarchy: {}", Error))),
					}
				},

				Err(_) => Ok(OkResponse(RequestId, &serde_json::Value::Array(Vec::new()))),
			}
		},

		"$provideTypeHierarchyItems" | "prepareTypeHierarchy" => {
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

					match Service.environment.PrepareTypeHierarchy(DocURI, Pos).await {
						Ok(Result) => Ok(OkResponse(RequestId, &Result)),

						Err(Error) => Ok(ErrResponse(RequestId, -32000, format!("prepareTypeHierarchy: {}", Error))),
					}
				},

				Err(_) => Ok(OkResponse(RequestId, &serde_json::Value::Array(Vec::new()))),
			}
		},

		// ---- Unknown ----
		_ => {
			dev_log!("cocoon", "warn: [CocoonService] Unknown generic method: {}", Req.method);

			Ok(ErrResponse(RequestId, -32601, format!("Method '{}' not found", Req.method)))
		},
	}
}
