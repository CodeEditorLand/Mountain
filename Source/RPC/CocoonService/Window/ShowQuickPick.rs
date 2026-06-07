//! Display a quick-pick UI through the `UserInterfaceProvider`. The
//! returned label strings are mapped back to indices via linear search
//! so the proto response can carry stable `selected_indices`.

use tonic::{Response, Status};

use CommonLibrary::UserInterface::{
	DTO::{QuickPickItemDTO::QuickPickItemDTO, QuickPickOptionsDTO::QuickPickOptionsDTO},
	UserInterfaceProvider::UserInterfaceProvider,
};

use ::Vine::Generated::{ShowQuickPickRequest, ShowQuickPickResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ShowQuickPickRequest,
) -> Result<Response<ShowQuickPickResponse>, Status> {

	dev_log!("cocoon", "[CocoonService] show_quick_pick: {} items", Request.items.len());

	let Items:Vec<QuickPickItemDTO> = Request
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
		Title:if Request.title.is_empty() { None } else { Some(Request.title.clone()) },
		PlaceHolder:if Request.placeholder.is_empty() {
			None
		} else {
			Some(Request.placeholder.clone())
		},
		CanPickMany:Some(Request.can_pick_many),
		IgnoreFocusOut:None,
	});

	match Service.environment.ShowQuickPick(Items, Options).await {
		Ok(Some(Selected)) => {
			let SelectedIndices:Vec<u32> = Selected
				.iter()
				.filter_map(|Label| {
					Request
						.items
						.iter()
						.position(|Item| &Item.label == Label)
						.map(|Index| Index as u32)
				})
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
