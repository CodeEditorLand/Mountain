//! Theme dispatcher.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::UI::{
	ThemesGetActive::Fn as ThemesGetActive,
	ThemesList::Fn as ThemesList,
	ThemesSet::Fn as ThemesSet,
};

/// Dispatches theme commands.
pub async fn dispatch_theme(
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"themes:getActive" | "themes:getColorTheme" => ThemesGetActive(runtime.clone()).await,

		"themes:list" => ThemesList(runtime.clone()).await,

		"themes:set" => ThemesSet(runtime.clone(), arguments).await,

		_ => Err(format!("Unknown theme command: {}", command)),
	}
}
