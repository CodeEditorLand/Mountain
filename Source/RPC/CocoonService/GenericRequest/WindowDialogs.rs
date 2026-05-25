//! Generic-request window/dialog handlers for `process_mountain_request`.
//! Handles `UserInterface.ShowOpenDialog`, `UserInterface.ShowSaveDialog`,
//! `UserInterface.ShowInputBox`, `showInformation`, `showWarning`, `showError`,
//! `showTextDocument`, `openDocument`, `createWebviewPanel`, `setWebviewHtml`,
//! `createStatusBarItem`, `setStatusBarText`, `saveAll`, `applyEdit`,
//! `openExternal`.

use CommonLibrary::UserInterface::UserInterfaceProvider::UserInterfaceProvider;
use serde_json::{Value, json};
use tauri::Emitter;
use tonic::Response;

use crate::{Environment::MountainEnvironment::MountainEnvironment, Vine::Generated::GenericResponse};
use super::FileSystem::{ErrResponse, OkResponse};

pub async fn HandleShowOpenDialog(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	use CommonLibrary::UserInterface::DTO::OpenDialogOptionsDTO::OpenDialogOptionsDTO;

	let Title = Params
		.get(0)
		.and_then(|V| V.get("title"))
		.and_then(|T| T.as_str())
		.map(|S| S.to_string());

	let Options = OpenDialogOptionsDTO {
		Base:CommonLibrary::UserInterface::DTO::DialogOptionsDTO::DialogOptionsDTO { Title, ..Default::default() },
		..OpenDialogOptionsDTO::default()
	};

	match Env.ShowOpenDialog(Some(Options)).await {
		Ok(Some(Paths)) => {
			let Uris:Vec<String> = Paths.iter().map(|P| format!("file://{}", P.display())).collect();

			OkResponse(RequestId, &json!(Uris))
		},

		Ok(None) => OkResponse(RequestId, &json!(serde_json::Value::Array(vec![]))),

		Err(Error) => ErrResponse(RequestId, -32000, Error.to_string()),
	}
}

pub async fn HandleShowSaveDialog(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	use CommonLibrary::UserInterface::DTO::SaveDialogOptionsDTO::SaveDialogOptionsDTO;

	let Title = Params
		.get(0)
		.and_then(|V| V.get("title"))
		.and_then(|T| T.as_str())
		.map(|S| S.to_string());

	let Options = SaveDialogOptionsDTO {
		Base:CommonLibrary::UserInterface::DTO::DialogOptionsDTO::DialogOptionsDTO { Title, ..Default::default() },
		..SaveDialogOptionsDTO::default()
	};

	match Env.ShowSaveDialog(Some(Options)).await {
		Ok(Some(Path)) => OkResponse(RequestId, &json!(format!("file://{}", Path.display()))),

		Ok(None) => OkResponse(RequestId, &Value::Null),

		Err(Error) => ErrResponse(RequestId, -32000, Error.to_string()),
	}
}

pub async fn HandleShowInputBox(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	use CommonLibrary::UserInterface::DTO::InputBoxOptionsDTO::InputBoxOptionsDTO;

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

	match Env.ShowInputBox(Some(Options)).await {
		Ok(Some(Text)) => OkResponse(RequestId, &json!(Text)),

		Ok(None) => OkResponse(RequestId, &Value::Null),

		Err(Error) => ErrResponse(RequestId, -32000, Error.to_string()),
	}
}

pub async fn HandleShowMessage(
	RequestId:u64,

	Params:Value,

	Env:&MountainEnvironment,

	Severity:CommonLibrary::UserInterface::DTO::MessageSeverity::MessageSeverity,
) -> Response<GenericResponse> {
	let Message = Params.get("message").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Items:Option<Value> = Params
		.get("items")
		.cloned()
		.filter(|V| V.is_array() && !V.as_array().unwrap().is_empty());

	match Env.ShowMessage(Severity, Message, Items).await {
		Ok(Some(Selected)) => OkResponse(RequestId, &json!({ "selectedItem": Selected })),

		Ok(None) => OkResponse(RequestId, &Value::Null),

		Err(Error) => ErrResponse(RequestId, -32000, Error.to_string()),
	}
}

pub fn HandleShowTextDocument(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let Uri = Params
		.get("uri")
		.and_then(|V| V.get("value").or(Some(V)))
		.and_then(|V| V.as_str())
		.unwrap_or("")
		.to_string();

	let ViewColumn = Params.get("viewColumn").and_then(|V| V.as_i64()).map(|N| N + 2);

	let PreserveFocus = Params.get("preserveFocus").and_then(|V| V.as_bool()).unwrap_or(false);

	let _ = Env.ApplicationHandle.emit(
		"sky://editor/openDocument",
		json!({ "uri": Uri, "viewColumn": ViewColumn, "preserveFocus": PreserveFocus }),
	);

	OkResponse(RequestId, &json!({ "success": true }))
}

pub fn HandleOpenDocument(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let Uri = Params
		.get("uri")
		.and_then(|V| V.get("value").or(Some(V)))
		.and_then(|V| V.as_str())
		.unwrap_or("")
		.to_string();

	let ViewColumn = Params.get("viewColumn").and_then(|V| V.as_i64());

	let _ = Env
		.ApplicationHandle
		.emit("sky://editor/openDocument", json!({ "uri": Uri, "viewColumn": ViewColumn }));

	OkResponse(RequestId, &json!({ "success": true }))
}

pub fn HandleSaveAll(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let IncludeUntitled = Params.get("includeUntitled").and_then(|V| V.as_bool()).unwrap_or(false);

	let _ = Env
		.ApplicationHandle
		.emit("sky://editor/saveAll", json!({ "includeUntitled": IncludeUntitled }));

	OkResponse(RequestId, &json!({ "success": true }))
}

pub fn HandleApplyEdit(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let Uri = Params
		.get("uri")
		.and_then(|V| V.get("value").or(Some(V)))
		.and_then(|V| V.as_str())
		.unwrap_or("")
		.to_string();

	let Edits = Params.get("edits").cloned().unwrap_or(json!([]));

	let _ = Env
		.ApplicationHandle
		.emit("sky://editor/applyEdits", json!({ "uri": Uri, "edits": Edits }));

	OkResponse(RequestId, &json!({ "success": true }))
}

pub fn HandleOpenExternal(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let Url = Params
		.as_str()
		.or_else(|| Params.get("url").and_then(|V| V.as_str()))
		.unwrap_or("")
		.to_string();

	let _ = Env.ApplicationHandle.emit("sky://native/openExternal", json!({ "url": Url }));

	OkResponse(RequestId, &json!({ "success": true }))
}

pub fn HandleCreateStatusBarItem(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Tooltip = Params.get("tooltip").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.ApplicationHandle.emit(
		"sky://statusbar/set-entry",
		json!({ "id": Id, "text": Text, "tooltip": Tooltip }),
	);

	OkResponse(RequestId, &json!({ "itemId": Id }))
}

pub fn HandleSetStatusBarText(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let ItemId = Params.get("itemId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://statusbar/update", json!({ "id": ItemId, "text": Text }));

	OkResponse(RequestId, &json!({ "success": true }))
}

pub fn HandleCreateWebviewPanel(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let ViewType = Params.get("viewType").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Title = Params.get("title").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Handle = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.map(|D| D.as_millis() as u64)
		.unwrap_or(0);

	let _ = Env.ApplicationHandle.emit(
		"sky://webview/create",
		json!({
			"handle": Handle,
			"viewType": ViewType,
			"title": Title,
			"viewColumn": Params.get("viewColumn"),
			"preserveFocus": Params.get("preserveFocus").and_then(|V| V.as_bool()).unwrap_or(false),
		}),
	);

	OkResponse(RequestId, &json!({ "handle": Handle }))
}

pub fn HandleSetWebviewHtml(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0);

	let Html = Params.get("html").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://webview/set-html", json!({ "handle": Handle, "html": Html }));

	OkResponse(RequestId, &json!({ "success": true }))
}
