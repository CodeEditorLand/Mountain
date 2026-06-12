//! Theme command router.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::UI::{
		ThemesGetActive::Fn as ThemesGetActive,
		ThemesList::Fn as ThemesList,
		ThemesSet::Fn as ThemesSet,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes theme commands. Returns Some(result) for handled commands,
/// None otherwise.
pub(crate) async fn route(
	_ApplicationHandle:tauri::AppHandle,

	RunTime:Arc<ApplicationRunTime>,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"themes:getActive" => {
			dev_log!("themes", "{}", command);

			Some(ThemesGetActive(RunTime.clone()).await)
		},

		"themes:list" => {
			dev_log!("themes", "{}", command);

			Some(ThemesList(RunTime.clone()).await)
		},

		"themes:set" => {
			dev_log!("themes", "{}", command);

			Some(ThemesSet(RunTime.clone(), Arguments).await)
		},

		"themes:getColorTheme" => {
			dev_log!("themes", "themes:getColorTheme (→ getActive)");

			Some(ThemesGetActive(RunTime.clone()).await)
		},

		_ => None,
	}
}
