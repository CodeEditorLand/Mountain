//! Wire method: `themes:list`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(_runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Themes = vec![
		json!({ "id": "Default Dark Modern", "label": "Default Dark Modern", "kind": "dark" }),
		json!({ "id": "Default Light Modern", "label": "Default Light Modern", "kind": "light" }),
		json!({ "id": "Default Dark+", "label": "Default Dark+", "kind": "dark" }),
		json!({ "id": "Default Light+", "label": "Default Light+", "kind": "light" }),
		json!({ "id": "High Contrast", "label": "High Contrast", "kind": "highContrast" }),
		json!({ "id": "High Contrast Light", "label": "High Contrast Light", "kind": "highContrastLight" }),
	];

	Ok(json!(Themes))
}
