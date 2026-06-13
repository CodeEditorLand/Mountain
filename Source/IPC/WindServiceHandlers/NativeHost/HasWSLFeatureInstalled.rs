//! Wire method: `nativeHost:hasWSLFeatureInstalled`.
//!
//! On Windows, checks if WSL is installed. Returns false on non-Windows.

use serde_json::{Value, json};

pub async fn Fn() -> Result<Value, String> {
	#[cfg(target_os = "windows")]
	{
		Ok(json!(std::path::Path::new("C:\\Windows\\System32\\wsl.exe").exists()))
	}

	#[cfg(not(target_os = "windows"))]
	{
		Ok(json!(false))
	}
}
