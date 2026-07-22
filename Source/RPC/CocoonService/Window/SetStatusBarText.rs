//! Update the text of a status-bar entry. Re-issues `SetStatusBarEntry`
//! so the stored DTO's `Text` field is refreshed in
//! `ActiveStatusBarItems` (HashMap insert acts as create-or-update).
use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use CommonLibrary::StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider};
use ::Vine::Generated::{Empty, SetStatusBarTextRequest};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(Service:&CocoonServiceImpl, Request:SetStatusBarTextRequest) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] set_status_bar_text: id={} text={}",
		Request.item_id,
		Request.text
	);

	let Entry = StatusBarEntryDTO {
		EntryIdentifier:Request.item_id.clone(),

		ItemIdentifier:Request.item_id.clone(),

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

	match Service.environment.SetStatusBarEntry(Entry).await {
		Ok(()) => {},

		Err(Error) => {
			dev_log!("cocoon", "warn: [CocoonService] set_status_bar_text trait failed: {}", Error);

			let _ = Service
				.environment
				.ApplicationHandle
				.emit("sky://statusbar/update", json!({ "id": Request.item_id, "text": Request.text }));
		},
	}

	Ok(Response::new(Empty {}))
}
