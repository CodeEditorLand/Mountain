//! Single diff target URI used by Wind's `--diff` launch
//! flag. Mirrors `vscode.IDiffEditorInput::resource`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub file_uri:String,
}
