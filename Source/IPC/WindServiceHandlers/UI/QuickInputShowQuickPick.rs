
//! Wire method: `quickInput:showQuickPick`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::UserInterface::{
		DTO::{QuickPickItemDTO::QuickPickItemDTO, QuickPickOptionsDTO::QuickPickOptionsDTO},
		UserInterfaceProvider::UserInterfaceProvider,
	};

	let Items:Vec<QuickPickItemDTO> = Arguments
		.first()
		.and_then(|V| V.as_array())
		.map(|Arr| {
			Arr.iter()
				.filter_map(|Item| {
					let Label = Item.get("label").and_then(|L| L.as_str()).unwrap_or("").to_string();
					let Description = Item.get("description").and_then(|D| D.as_str()).map(|S| S.to_string());
					let Detail = Item.get("detail").and_then(|D| D.as_str()).map(|S| S.to_string());
					let Picked = Item.get("picked").and_then(|P| P.as_bool()).unwrap_or(false);
					Some(QuickPickItemDTO { Label, Description, Detail, Picked:Some(Picked), AlwaysShow:Some(false) })
				})
				.collect()
		})
		.unwrap_or_default();

	let Options = QuickPickOptionsDTO {
		PlaceHolder:Arguments
			.get(1)
			.and_then(|V| V.get("placeholder"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),

		CanPickMany:Some(
			Arguments
				.get(1)
				.and_then(|V| V.get("canPickMany"))
				.and_then(|B| B.as_bool())
				.unwrap_or(false),
		),

		Title:Arguments
			.get(1)
			.and_then(|V| V.get("title"))
			.and_then(|T| T.as_str())
			.map(|S| S.to_string()),
		..Default::default()
	};

	let Result = RunTime
		.Environment
		.ShowQuickPick(Items, Some(Options))
		.await
		.map_err(|Error| format!("quickInput:showQuickPick failed: {}", Error))?;

	match Result {
		Some(Labels) => Ok(Labels.into_iter().next().map(|S| json!(S)).unwrap_or(Value::Null)),

		None => Ok(Value::Null),
	}
}
