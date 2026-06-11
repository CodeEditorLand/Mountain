//! `extensions:getManifest` IPC handler - reads the full manifest out of
//! a VSIX archive on disk (no install side effects). Used by VS Code's
//! "Install from VSIX" preview and by the extension-details page before
//! an install is confirmed. Accepts a bare path string or a serialised
//! URI object (`fsPath`/`path`).

use serde_json::Value;

use crate::dev_log;

pub async fn Fn(Args:Vec<Value>) -> Result<Value, String> {
	let VsixPath = match Args.first() {
		Some(Value::String(Path)) => Path.clone(),
		Some(Obj) => {
			Obj.get("fsPath")
				.and_then(|V| V.as_str())
				.map(str::to_owned)
				.or_else(|| Obj.get("path").and_then(|V| V.as_str()).map(str::to_owned))
				.unwrap_or_default()
		},
		None => String::new(),
	};

	dev_log!("extensions", "extensions:getManifest vsix={}", VsixPath);

	if VsixPath.is_empty() {
		return Err("extensions:getManifest: missing VSIX path argument".to_string());
	}

	let Path = std::path::PathBuf::from(&VsixPath);

	match crate::ExtensionManagement::VsixInstaller::ReadFullManifest(&Path) {
		Ok(Manifest) => Ok(Manifest),
		Err(Error) => {
			dev_log!(
				"extensions",
				"warn: [ExtensionGetManifest] extensions:getManifest failed for '{}': {}",
				VsixPath,
				Error
			);

			Err(format!("extensions:getManifest failed: {}", Error))
		},
	}
}
