use CommonLibrary::IPC::DTO::ProxyTarget::ProxyTarget;
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	Track::Effect::{
		CreateEffectForRequest::Utilities::{
			Params::ValAt,
			Proxy::crate::Track::Effect::CreateEffectForRequest::Utilities::Proxy::Fn,
		},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Task.Fetch" => {
			crate::effect!(RunTime, {
				let Filter = ValAt(&Parameters, 0);
				crate::Track::Effect::CreateEffectForRequest::Utilities::Proxy::Fn(
					&RunTime,
					ProxyTarget::ExtHostTaskService,
					"fetchTasks",
					json!([filter]),
					5000,
				)
				.await
				.or_else(|Error| {
					dev_log!("ipc", "warn: [Task.Fetch] extension did not answer ({:?}); returning []", error);
					Ok(json!([]))
				})
			})
		},

		"Task.Execute" => {
			crate::effect!(RunTime, {
				let Task = ValAt(&Parameters, 0);
				crate::Track::Effect::CreateEffectForRequest::Utilities::Proxy::Fn(
					&RunTime,
					ProxyTarget::ExtHostTaskService,
					"executeTask",
					json!([task]),
					30000,
				)
				.await
				.or_else(|Error| {
					dev_log!(
						"ipc",
						"warn: [Task.Execute] extension did not answer ({:?}); returning null",
						error
					);
					Ok(json!(null))
				})
			})
		},

		_ => None,
	}
}
