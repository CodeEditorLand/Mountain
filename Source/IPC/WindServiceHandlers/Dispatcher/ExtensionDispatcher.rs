//! Extension management command dispatcher.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Extension::{
	ExtensionInstall::Fn as ExtensionInstall,
	ExtensionUninstall::Fn as ExtensionUninstall,
	VsixPathFromArgs::Fn as VsixPathFromArgs,
};

/// Dispatches extension commands.
///
/// Handled commands:
/// - `extensions:install`
/// - `extensions:uninstall`
/// - `extensions:getManifest`
pub async fn dispatch_extension(
	app_handle:&tauri::AppHandle,

	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"extensions:install" => ExtensionInstall(app_handle.clone(), runtime.clone(), arguments).await,

		"extensions:uninstall" => ExtensionUninstall(app_handle.clone(), runtime.clone(), arguments).await,

		"extensions:getManifest" => {
			let vsix_path = match arguments.first() {
				Some(serde_json::Value::String(path)) => path.clone(),

				Some(obj) => {
					obj.get("fsPath")
						.and_then(|v| v.as_str())
						.map(str::to_owned)
						.or_else(|| obj.get("path").and_then(|v| v.as_str()).map(str::to_owned))
						.unwrap_or_default()
				},

				None => String::new(),
			};

			if vsix_path.is_empty() {
				Err("extensions:getManifest: missing VSIX path argument".to_string())
			} else {
				let path = std::path::PathBuf::from(&vsix_path);

				match crate::ExtensionManagement::VsixInstaller::ReadFullManifest(&path) {
					Ok(manifest) => Ok(manifest),

					Err(e) => Err(format!("extensions:getManifest failed: {}", e)),
				}
			}
		},

		"extensions:reinstall" | "extensions:updateMetadata" => {
			crate::dev_log!("extensions", "{} (no-op: no gallery backend)", command);

			Ok(Value::Null)
		},

		_ => Err(format!("Unknown extension command: {}", command)),
	}
}
