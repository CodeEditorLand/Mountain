#![allow(non_snake_case, unused_variables)]
//! QuickPick / InputBox dialog handlers. Routes through
//! `UserInterfaceProvider` so the actual dialog rendering stays
//! platform-agnostic (Tauri-webview on desktop; extensible to a future
//! browser preview).

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn QuickInputShowQuickPick(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
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

pub async fn QuickInputShowInputBox(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::UserInterface::{
		DTO::InputBoxOptionsDTO::InputBoxOptionsDTO,
		UserInterfaceProvider::UserInterfaceProvider,
	};

	let Opts = Arguments.first();

	let Options = InputBoxOptionsDTO {
		Prompt:Opts
			.and_then(|V| V.get("prompt"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),

		PlaceHolder:Opts
			.and_then(|V| V.get("placeholder"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),

		IsPassword:Some(Opts.and_then(|V| V.get("password")).and_then(|B| B.as_bool()).unwrap_or(false)),

		Value:Opts
			.and_then(|V| V.get("value"))
			.and_then(|V| V.as_str())
			.map(|S| S.to_string()),

		Title:Opts
			.and_then(|V| V.get("title"))
			.and_then(|T| T.as_str())
			.map(|S| S.to_string()),

		IgnoreFocusOut:None,
	};

	let Result = RunTime
		.Environment
		.ShowInputBox(Some(Options))
		.await
		.map_err(|Error| format!("quickInput:showInputBox failed: {}", Error))?;

	Ok(Result.map(|S| json!(S)).unwrap_or(Value::Null))
}
