//! QuickInput command dispatcher.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::UI::{
	QuickInputShowInputBox::Fn as QuickInputShowInputBox,
	QuickInputShowQuickPick::Fn as QuickInputShowQuickPick,
};

/// Dispatches quick input commands.
///
/// Handled commands:
/// - `quickInput:showQuickPick`
/// - `quickInput:showInputBox`
pub async fn dispatch_quick_input(
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"quickInput:showQuickPick" => QuickInputShowQuickPick(runtime.clone(), arguments).await,

		"quickInput:showInputBox" => QuickInputShowInputBox(runtime.clone(), arguments).await,

		_ => Err(format!("Unknown quick input command: {}", command)),
	}
}
