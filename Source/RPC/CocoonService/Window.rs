#![allow(non_snake_case)]
//! Window domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: show_text_document, show_information_message,
//! show_warning_message, show_error_message, create_status_bar_item,
//! set_status_bar_text, create_webview_panel, set_webview_html,
//! on_did_receive_message, post_webview_message, dispose_webview_panel,
//! open_external, show_quick_pick, show_input_box, show_progress,
//! report_progress.

use std::time::{SystemTime, UNIX_EPOCH};

use CommonLibrary::UserInterface::DTO::{
	InputBoxOptionsDTO::InputBoxOptionsDTO,
	QuickPickItemDTO::QuickPickItemDTO,
	QuickPickOptionsDTO::QuickPickOptionsDTO,
};
use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::{
	Vine::Generated::{
		CreateStatusBarItemRequest,
		CreateStatusBarItemResponse,
		CreateWebviewPanelRequest,
		CreateWebviewPanelResponse,
		DisposeWebviewPanelRequest,
		Empty,
		OnDidReceiveMessageRequest,
		OpenExternalRequest,
		PostWebviewMessageRequest,
		ReportProgressRequest,
		SetStatusBarTextRequest,
		SetWebviewHtmlRequest,
		ShowInputBoxRequest,
		ShowInputBoxResponse,
		ShowMessageRequest,
		ShowMessageResponse,
		ShowProgressRequest,
		ShowProgressResponse,
		ShowQuickPickRequest,
		ShowQuickPickResponse,
		ShowTextDocumentRequest,
		ShowTextDocumentResponse,
		on_did_receive_message_request,
		post_webview_message_request,
	},
	dev_log,
};

pub async fn ShowTextDocument(
	Service:&CocoonServiceImpl,
	req:ShowTextDocumentRequest,
) -> Result<Response<ShowTextDocumentResponse>, Status> {
	let Uri = req.uri.as_ref().map(|U| U.value.clone()).unwrap_or_default();
	dev_log!("cocoon", "[CocoonService] show_text_document: {}", Uri);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://editor/openDocument",
		json!({ "uri": Uri, "viewColumn": req.view_column }),
	);

	Ok(Response::new(ShowTextDocumentResponse { success:true }))
}

pub async fn ShowInformationMessage(
	Service:&CocoonServiceImpl,
	req:ShowMessageRequest,
) -> Result<Response<ShowMessageResponse>, Status> {
	use CommonLibrary::UserInterface::{
		DTO::MessageSeverity::MessageSeverity,
		UserInterfaceProvider::UserInterfaceProvider,
	};

	dev_log!("cocoon", "[CocoonService] show_information_message: {}", req.message);

	let _ = Service.environment.ShowMessage(MessageSeverity::Info, req.message, None).await;

	Ok(Response::new(ShowMessageResponse { success:true }))
}

pub async fn ShowWarningMessage(
	Service:&CocoonServiceImpl,
	req:ShowMessageRequest,
) -> Result<Response<ShowMessageResponse>, Status> {
	use CommonLibrary::UserInterface::{
		DTO::MessageSeverity::MessageSeverity,
		UserInterfaceProvider::UserInterfaceProvider,
	};

	dev_log!("cocoon", "warn: [CocoonService] show_warning_message: {}", req.message);

	let _ = Service
		.environment
		.ShowMessage(MessageSeverity::Warning, req.message, None)
		.await;

	Ok(Response::new(ShowMessageResponse { success:true }))
}

pub async fn ShowErrorMessage(
	Service:&CocoonServiceImpl,
	req:ShowMessageRequest,
) -> Result<Response<ShowMessageResponse>, Status> {
	use CommonLibrary::UserInterface::{
		DTO::MessageSeverity::MessageSeverity,
		UserInterfaceProvider::UserInterfaceProvider,
	};

	dev_log!("cocoon", "error: [CocoonService] show_error_message: {}", req.message);

	let _ = Service.environment.ShowMessage(MessageSeverity::Error, req.message, None).await;

	Ok(Response::new(ShowMessageResponse { success:true }))
}

pub async fn CreateStatusBarItem(
	Service:&CocoonServiceImpl,
	req:CreateStatusBarItemRequest,
) -> Result<Response<CreateStatusBarItemResponse>, Status> {
	use CommonLibrary::StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider};
	dev_log!("cocoon", "[CocoonService] create_status_bar_item: {}", req.id);

	// Delegate to the StatusBarProvider trait. The trait impl inserts
	// the entry into `ApplicationState::Feature::Markers::ActiveStatusBarItems`
	// keyed on `EntryIdentifier`; without this registration the
	// workbench has no memory of the entry and the first
	// `set_status_bar_text` call with the same id rebroadcasts a
	// fresh entry (leaking state). The trait emits
	// `SkyEvent::StatusBarSetEntry` internally, so the UI still
	// sees the new item.
	let Entry = StatusBarEntryDTO {
		EntryIdentifier:req.id.clone(),
		ItemIdentifier:req.id.clone(),
		ExtensionIdentifier:String::new(),
		Name:None,
		Text:req.text.clone(),
		Tooltip:if req.tooltip.is_empty() { None } else { Some(json!(req.tooltip)) },
		HasTooltipProvider:false,
		Command:None,
		Color:None,
		BackgroundColor:None,
		IsAlignedLeft:true,
		Priority:None,
		AccessibilityInformation:None,
	};
	if let Err(Error) = Service.environment.SetStatusBarEntry(Entry).await {
		dev_log!("cocoon", "warn: [CocoonService] create_status_bar_item trait failed: {}", Error);
		// Fallback to direct Sky emit so the UI still gets an update.
		let _ = Service.environment.ApplicationHandle.emit(
			"sky://statusbar/create",
			json!({ "id": req.id, "text": req.text, "tooltip": req.tooltip }),
		);
	}

	Ok(Response::new(CreateStatusBarItemResponse { item_id:req.id.clone() }))
}

pub async fn SetStatusBarText(
	Service:&CocoonServiceImpl,
	req:SetStatusBarTextRequest,
) -> Result<Response<Empty>, Status> {
	use CommonLibrary::StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider};
	dev_log!(
		"cocoon",
		"[CocoonService] set_status_bar_text: id={} text={}",
		req.item_id,
		req.text
	);

	// Re-set via `SetStatusBarEntry` so the stored entry's Text
	// field is actually updated in `ActiveStatusBarItems`, not just
	// emitted to Sky and lost. The trait handles
	// create-or-update semantics by HashMap insert.
	let Entry = StatusBarEntryDTO {
		EntryIdentifier:req.item_id.clone(),
		ItemIdentifier:req.item_id.clone(),
		ExtensionIdentifier:String::new(),
		Name:None,
		Text:req.text.clone(),
		Tooltip:None,
		HasTooltipProvider:false,
		Command:None,
		Color:None,
		BackgroundColor:None,
		IsAlignedLeft:true,
		Priority:None,
		AccessibilityInformation:None,
	};
	if let Err(Error) = Service.environment.SetStatusBarEntry(Entry).await {
		dev_log!("cocoon", "warn: [CocoonService] set_status_bar_text trait failed: {}", Error);
		let _ = Service
			.environment
			.ApplicationHandle
			.emit("sky://statusbar/update", json!({ "id": req.item_id, "text": req.text }));
	}

	Ok(Response::new(Empty {}))
}

pub async fn CreateWebviewPanel(
	Service:&CocoonServiceImpl,
	req:CreateWebviewPanelRequest,
) -> Result<Response<CreateWebviewPanelResponse>, Status> {
	use CommonLibrary::Webview::WebviewProvider::WebviewProvider;
	dev_log!(
		"cocoon",
		"[CocoonService] create_webview_panel: view_type={} title={}",
		req.view_type,
		req.title
	);

	// Delegate to the trait impl so the panel is registered in
	// `ApplicationState::WebviewState`. Without this the extension
	// gets a handle but Mountain never knows about the panel; the
	// first `DisposeWebviewPanel` call then fails with "unknown
	// handle" and webviews leak DOM into the workbench. The trait
	// impl emits the `sky://webview/create` event internally.
	let Handle = match Service
		.environment
		.CreateWebviewPanel(
			json!({}),
			req.view_type.clone(),
			req.title.clone(),
			json!({ "viewColumn": req.view_column, "preserveFocus": req.preserve_focus }),
			json!({}),
			json!({}),
		)
		.await
	{
		Ok(H) => H,
		Err(Error) => {
			dev_log!("cocoon", "warn: [CocoonService] create_webview_panel trait failed: {}", Error);
			// Fallback to a millisecond handle so the extension gets
			// *something* back. The legacy direct Sky emit preserves
			// existing behaviour for callers that don't care about
			// state tracking.
			let Fallback = SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.map(|D| D.as_millis() as u32)
				.unwrap_or(0);
			let _ = Service.environment.ApplicationHandle.emit(
				"sky://webview/create",
				json!({
					"handle": Fallback,
					"viewType": req.view_type,
					"title": req.title,
					"viewColumn": req.view_column,
					"preserveFocus": req.preserve_focus,
					"iconPath": req.icon_path,
				}),
			);
			return Ok(Response::new(CreateWebviewPanelResponse { handle:Fallback }));
		},
	};

	let HandleU32 = Handle.parse::<u32>().unwrap_or_else(|_| {
		// The trait returns a numeric string; collapse to a hash if
		// it isn't parseable so the proto field (u32) still gets a
		// stable value.
		Handle.chars().map(|C| C as u32).fold(0u32, |A, C| A.wrapping_add(C))
	});

	Ok(Response::new(CreateWebviewPanelResponse { handle:HandleU32 }))
}

pub async fn SetWebviewHtml(Service:&CocoonServiceImpl, req:SetWebviewHtmlRequest) -> Result<Response<Empty>, Status> {
	use CommonLibrary::Webview::WebviewProvider::WebviewProvider;
	dev_log!(
		"cocoon",
		"[CocoonService] set_webview_html: handle={} ({} bytes)",
		req.handle,
		req.html.len()
	);

	// Delegate to trait so the HTML content is captured in
	// `WebviewStateDTO` before the Sky emit. Subsequent reveal /
	// restore operations can re-serve the content without a
	// re-issue from the extension. Trait impl also emits
	// `sky://webview/set-html` internally.
	if let Err(Error) = Service
		.environment
		.SetWebviewHTML(req.handle.to_string(), req.html.clone())
		.await
	{
		dev_log!("cocoon", "warn: [CocoonService] set_webview_html trait failed: {}", Error);
		// Fallback: emit directly so the panel still sees the update.
		let _ = Service
			.environment
			.ApplicationHandle
			.emit("sky://webview/set-html", json!({ "handle": req.handle, "html": req.html }));
	}

	Ok(Response::new(Empty {}))
}

pub async fn OnDidReceiveMessage(
	Service:&CocoonServiceImpl,
	req:OnDidReceiveMessageRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] on_did_receive_message: handle={}", req.handle);

	let MessagePayload = match &req.message {
		Some(on_did_receive_message_request::Message::StringMessage(S)) => json!(S),
		Some(on_did_receive_message_request::Message::BytesMessage(B)) => json!(B),
		None => serde_json::Value::Null,
	};

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://webview/message",
		json!({ "handle": req.handle, "message": MessagePayload }),
	);

	Ok(Response::new(Empty {}))
}

pub async fn PostWebviewMessage(
	Service:&CocoonServiceImpl,
	req:PostWebviewMessageRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] post_webview_message: handle={}", req.handle);

	let MessagePayload = match &req.message {
		Some(post_webview_message_request::Message::StringMessage(S)) => json!(S),
		Some(post_webview_message_request::Message::BytesMessage(B)) => json!(B),
		None => serde_json::Value::Null,
	};

	// Canonical kebab-case channel; `sky://webview/postMessage` has been retired.
	let _ = Service.environment.ApplicationHandle.emit(
		"sky://webview/post-message",
		json!({
			"handle": req.handle,
			"message": MessagePayload,
		}),
	);

	Ok(Response::new(Empty {}))
}

pub async fn DisposeWebviewPanel(
	Service:&CocoonServiceImpl,
	req:DisposeWebviewPanelRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] dispose_webview_panel: handle={}", req.handle);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://webview/dispose", json!({ "handle": req.handle }));

	Ok(Response::new(Empty {}))
}

pub async fn OpenExternal(Service:&CocoonServiceImpl, req:OpenExternalRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] open_external: {}", req.uri);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://native/openExternal", json!({ "url": req.uri }));

	Ok(Response::new(Empty {}))
}

pub async fn ShowQuickPick(
	Service:&CocoonServiceImpl,
	req:ShowQuickPickRequest,
) -> Result<Response<ShowQuickPickResponse>, Status> {
	use CommonLibrary::UserInterface::UserInterfaceProvider::UserInterfaceProvider;

	dev_log!("cocoon", "[CocoonService] show_quick_pick: {} items", req.items.len());

	let Items:Vec<QuickPickItemDTO> = req
		.items
		.iter()
		.map(|Item| {
			QuickPickItemDTO {
				Label:Item.label.clone(),
				Description:if Item.description.is_empty() { None } else { Some(Item.description.clone()) },
				Detail:None,
				Picked:Some(Item.picked),
				AlwaysShow:None,
			}
		})
		.collect();

	let Options = Some(QuickPickOptionsDTO {
		Title:if req.title.is_empty() { None } else { Some(req.title.clone()) },
		PlaceHolder:if req.placeholder.is_empty() { None } else { Some(req.placeholder.clone()) },
		CanPickMany:Some(req.can_pick_many),
		IgnoreFocusOut:None,
	});

	match Service.environment.ShowQuickPick(Items, Options).await {
		Ok(Some(Selected)) => {
			// Map selected label strings back to indices via linear search
			let SelectedIndices:Vec<u32> = Selected
				.iter()
				.filter_map(|Label| req.items.iter().position(|Item| &Item.label == Label).map(|Idx| Idx as u32))
				.collect();
			Ok(Response::new(ShowQuickPickResponse { selected_indices:SelectedIndices }))
		},
		Ok(None) => Ok(Response::new(ShowQuickPickResponse::default())),
		Err(Error) => {
			dev_log!("cocoon", "warn: [CocoonService] show_quick_pick failed: {}", Error);
			Ok(Response::new(ShowQuickPickResponse::default()))
		},
	}
}

pub async fn ShowInputBox(
	Service:&CocoonServiceImpl,
	req:ShowInputBoxRequest,
) -> Result<Response<ShowInputBoxResponse>, Status> {
	use CommonLibrary::UserInterface::UserInterfaceProvider::UserInterfaceProvider;

	dev_log!("cocoon", "[CocoonService] show_input_box");

	let Options = Some(InputBoxOptionsDTO {
		Title:if req.title.is_empty() { None } else { Some(req.title) },
		PlaceHolder:if req.placeholder.is_empty() { None } else { Some(req.placeholder) },
		Value:if req.value.is_empty() { None } else { Some(req.value) },
		Prompt:if req.prompt.is_empty() { None } else { Some(req.prompt) },
		IsPassword:if req.password { Some(true) } else { None },
		IgnoreFocusOut:None,
	});

	match Service.environment.ShowInputBox(Options).await {
		Ok(Some(Value)) => Ok(Response::new(ShowInputBoxResponse { value:Value, cancelled:false })),
		Ok(None) => Ok(Response::new(ShowInputBoxResponse { value:String::new(), cancelled:true })),
		Err(Error) => {
			dev_log!("cocoon", "warn: [CocoonService] show_input_box failed: {}", Error);
			Ok(Response::new(ShowInputBoxResponse { value:String::new(), cancelled:true }))
		},
	}
}

pub async fn ShowProgress(
	Service:&CocoonServiceImpl,
	req:ShowProgressRequest,
) -> Result<Response<ShowProgressResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] show_progress: title={}", req.title);

	let Handle = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|D| D.as_millis() as u32)
		.unwrap_or(0);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://progress/start",
		json!({
			"handle": Handle,
			"title": req.title,
			"cancellable": req.cancellable,
			"location": req.location,
		}),
	);

	Ok(Response::new(ShowProgressResponse { handle:Handle }))
}

pub async fn ReportProgress(Service:&CocoonServiceImpl, req:ReportProgressRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] report_progress: handle={}", req.handle);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://progress/update",
		json!({
			"handle": req.handle,
			"message": req.message,
			"increment": req.increment,
		}),
	);

	Ok(Response::new(Empty {}))
}
