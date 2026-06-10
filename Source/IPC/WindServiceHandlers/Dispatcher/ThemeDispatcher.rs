//! Theme dispatcher.

use serde_json::Value;

<<<<<<< HEAD
use crate::UI::{ThemesGetActive::Fn as ThemesGetActive, ThemesList::Fn as ThemesList, ThemesSet::Fn as ThemesSet};

/// Dispatches theme commands.
pub async fn dispatch_theme(
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,
=======
use crate::IPC::WindServiceHandlers::UI::{
	ThemesGetActive::Fn as ThemesGetActive,
	ThemesList::Fn as ThemesList,
	ThemesSet::Fn as ThemesSet,
};

/// Dispatches theme commands.
pub async fn dispatch_theme(
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

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
