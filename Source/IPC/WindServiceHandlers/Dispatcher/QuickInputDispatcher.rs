//! QuickInput command dispatcher.

use serde_json::Value;

<<<<<<< HEAD
use crate::UI::{
=======
use crate::IPC::WindServiceHandlers::UI::{
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
	QuickInputShowInputBox::Fn as QuickInputShowInputBox,
	QuickInputShowQuickPick::Fn as QuickInputShowQuickPick,
};

/// Dispatches quick input commands.
///
/// Handled commands:
/// - `quickInput:showQuickPick`
/// - `quickInput:showInputBox`
pub async fn dispatch_quick_input(
<<<<<<< HEAD
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,
=======
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"quickInput:showQuickPick" => QuickInputShowQuickPick(runtime.clone(), arguments).await,

		"quickInput:showInputBox" => QuickInputShowInputBox(runtime.clone(), arguments).await,

		_ => Err(format!("Unknown quick input command: {}", command)),
	}
}
