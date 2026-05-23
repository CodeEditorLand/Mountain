
//! Wire method: `quickInput:showInputBox`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
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
