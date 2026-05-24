//! Register a status-bar entry through the `StatusBarProvider` trait so
//! the entry lives in
//! `ApplicationState::Feature::Markers::ActiveStatusBarItems`. Without this
//! registration the workbench has no memory of the entry and
//! the first `SetStatusBarText::Fn` call rebroadcasts a fresh entry
//! (state leak). Falls back to a direct Sky emit on trait failure.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use CommonLibrary::StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{CreateStatusBarItemRequest, CreateStatusBarItemResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:CreateStatusBarItemRequest,
) -> Result<Response<CreateStatusBarItemResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] create_status_bar_item: {}", Request.id);

	let Entry = StatusBarEntryDTO {
		EntryIdentifier:Request.id.clone(),

		ItemIdentifier:Request.id.clone(),

		ExtensionIdentifier:String::new(),

		Name:None,

		Text:Request.text.clone(),

		Tooltip:if Request.tooltip.is_empty() { None } else { Some(json!(Request.tooltip)) },

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

		let _ = Service.environment.ApplicationHandle.emit(
			"sky://statusbar/create",
			json!({ "id": Request.id, "text": Request.text, "tooltip": Request.tooltip }),
		);
	}

	Ok(Response::new(CreateStatusBarItemResponse { ItemId:Request.id.clone() }))
}
