use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use super::{
	super::Internal::{AnalyzeTextLinesAndEol, UrlSerdeHelper},
	RpcModelContentChangeDto::RpcModelContentChangeDto,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentStateDto {
	// ... fields from provided source ...
}

impl DocumentStateDto {
	pub fn GetText(&self) -> String { self.Lines.join(&self.Eol) }

	// A more realistic ApplyChanges implementation
	pub fn ApplyChanges(&mut self, NewVersion:i64, ChangesValue:&Value) -> Result<(), String> {
		// ...
		Ok(())
	}
	// ... other methods
}
