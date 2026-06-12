//! Label command router.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Navigation::{
		LabelGetBase::Fn as LabelGetBase,
		LabelGetURI::Fn as LabelGetURI,
		LabelGetWorkspace::Fn as LabelGetWorkspace,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes label commands. Returns Some(result) for handled commands,
/// None otherwise.
pub(crate) async fn route(
	RunTime: Arc<ApplicationRunTime>,

	command: &str,

	Arguments: Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"label:getUri" => {
			dev_log!("label", "label:getUri");

			Some(LabelGetURI(RunTime.clone(), Arguments).await)
		},

		"label:getWorkspace" => {
			dev_log!("label", "label:getWorkspace");

			Some(LabelGetWorkspace(RunTime.clone()).await)
		},

		"label:getBase" => {
			dev_log!("label", "label:getBase");

			Some(LabelGetBase(Arguments).await)
		},

		_ => None,
	}
}
