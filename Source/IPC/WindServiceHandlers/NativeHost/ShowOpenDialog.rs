//! Wire method: `nativeHost:showOpenDialog`.
//!
//! Delegates to `NativeDialog::ShowOpenDialog::ShowOpenDialog`. This atom
//! preserves the stable handler name while the actual file/folder picker
//! lives under the NativeDialog domain (filter parsing, DialogFilter DTO,
//! etc. each in their own atom). Prior stub returned `canceled:true`, which
//! silently broke "Install from VSIX…"; delegation restores correctness.

use serde_json::Value;
use tauri::AppHandle;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	crate::IPC::WindServiceHandlers::NativeDialog::ShowOpenDialog::Fn(ApplicationHandle, Arguments).await
}
