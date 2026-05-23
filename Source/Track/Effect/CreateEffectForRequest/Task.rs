use CommonLibrary::IPC::DTO::ProxyTarget::ProxyTarget;
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	Track::Effect::{
		CreateEffectForRequest::Utilities::{Params::val_at, Proxy::proxy_cocoon},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Task.Fetch" => {
			crate::effect!(run_time, {
				let filter = val_at(&Parameters, 0);
				proxy_cocoon(&run_time, ProxyTarget::ExtHostTaskService, "fetchTasks", json!([filter]), 5000)
					.await
					.or_else(|error| {
						dev_log!("ipc", "warn: [Task.Fetch] extension did not answer ({:?}); returning []", error);
						Ok(json!([]))
					})
			})
		},

		"Task.Execute" => {
			crate::effect!(run_time, {
				let task = val_at(&Parameters, 0);
				proxy_cocoon(&run_time, ProxyTarget::ExtHostTaskService, "executeTask", json!([task]), 30000)
					.await
					.or_else(|error| {
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
