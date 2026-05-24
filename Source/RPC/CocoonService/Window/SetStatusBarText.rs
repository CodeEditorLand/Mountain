//! Update the text of a status-bar entry. Re-issues `SetStatusBarEntry`
//! so the stored DTO's `Text` field is refreshed in
//! `ActiveStatusBarItems` (HashMap insert acts as create-or-update).

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use CommonLibrary::StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, SetStatusBarTextRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:SetStatusBarTextRequest) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] set_status_bar_text: id={} text={}",
		Request.ItemId,
		Request.text
	);

	let Entry = StatusBarEntryDTO {
		EntryIdentifier:Request.ItemId.clone(),

		ItemIdentifier:Request.ItemId.clone(),

		ExtensionIdentifier:String::new(),

		Name:None,

		Text:Request.text.clone(),

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
			.emit("sky://statusbar/update", json!({ "id": Request.ItemId, "text": Request.text }));
	}

	Ok(Response::new(Empty {}))
}
