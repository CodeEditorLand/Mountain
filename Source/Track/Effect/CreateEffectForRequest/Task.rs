pub fn Matches(MethodName:&str) -> bool {
	match MethodName {
		"Task.Fetch" | "Task.Execute" => true,
		_ => false,
	}
}

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
pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:&Value) -> Option<Result<MappedEffect, String>> {
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

		// Cocoon's `Tasks/Namespace.ts:104` sends `terminate_task` when an
		// extension calls `vscode.tasks.executeTask(...).terminate()` on
		// the returned TaskExecution. Forward to the extension host so the
		// task provider can stop the underlying process. Treated as
		// best-effort: a missing/dead task provider should not throw,
		// since the task may have already exited.
		"terminate_task" | "Task.Terminate" => {
			crate::effect!(run_time, {
				let execution = val_at(&Parameters, 0);

				proxy_cocoon(
					&run_time,
					ProxyTarget::ExtHostTaskService,
					"terminateTask",
					json!([execution]),
					5000,
				)
				.await
				.or_else(|error| {
					dev_log!(
						"ipc",
						"warn: [Task.Terminate] extension did not answer ({:?}); treating as no-op",
						error
					);

					Ok(json!(null))
				})
			})
		},

		_ => None,
	}
}
