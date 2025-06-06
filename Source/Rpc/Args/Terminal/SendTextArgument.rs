// File: Rpc/Args/Terminal/SendTextArgument.rs

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SendTextArgument {
	// The unique numeric identifier of the target terminal instance.
	pub Id:u64,
	// The text string to send to the terminal as input.
	pub Text:String,
	// Optional. If true (default), a newline character will be appended to the text
	// before sending it to the terminal process.
	#[serde(alias = "addNewLine")]
	pub AddNewLine:Option<bool>,
}
