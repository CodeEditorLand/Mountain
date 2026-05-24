use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider,
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	Track::Effect::{
		CreateEffectForRequest::Utilities::Params::{I64At, ValAt},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$scm:createSourceControl" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn SourceControlManagementProvider> = RunTime.Environment.Require();
				let resource = ValAt(&Parameters, 0);
				provider
					.CreateSourceControl(resource)
					.await
					.map(|handle| json!(handle))
					.map_err(|E| e.to_string())
			})
		},

		"$scm:updateSourceControl" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn SourceControlManagementProvider> = RunTime.Environment.Require();
				let Handle = I64At(&Parameters, 0) as u32;
				let update = ValAt(&Parameters, 1);
				provider
					.UpdateSourceControl(handle, update)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"$scm:updateGroup" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn SourceControlManagementProvider> = RunTime.Environment.Require();
				let Handle = I64At(&Parameters, 0) as u32;
				let group_data = ValAt(&Parameters, 1);
				provider
					.UpdateSourceControlGroup(handle, group_data)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"$scm:registerInputBox" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn SourceControlManagementProvider> = RunTime.Environment.Require();
				let Handle = I64At(&Parameters, 0) as u32;
				let Options = ValAt(&Parameters, 1);
				provider
					.RegisterInputBox(handle, options)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		// `vscode.diff` is the canonical command the vscode.git extension
		// calls when the user clicks a staged or unstaged file in the SCM
		// sidebar. It receives three positional args:
		//   [0] leftUri   - the "before" URI  (e.g. git://…?HEAD)
		//   [1] rightUri  - the "after"  URI  (e.g. the working-tree file)
		//   [2] title     - string label shown in the editor tab
		//
		// Without this arm the command falls through to the Unknown-method
		// error branch, Mountain logs a warn, the extension's awaited
		// promise rejects, and the diff editor never opens.
		//
		// We forward the full parameter array to Sky on
		// `sky://editor/diff` as a round-trip request so the extension's
		// `await commands.executeCommand("vscode.diff", …)` resolves when
		// the workbench has actually opened the diff editor.
		//
		// `$scm:openDiff` is an older alias emitted by some extension
		// versions; we handle it identically.
		"vscode.diff" | "$scm:openDiff" => {
			crate::effect!(RunTime, {
				dev_log!(
					"scm",
					"[SCM] vscode.diff forwarding to sky://editor/diff params={:?}",
					Parameters
				);

				match crate::Environment::UserInterfaceProvider::SendUserInterfaceRequest(
					&RunTime.Environment,
					"sky://editor/diff",
					Parameters,
				)
				.await
				{
					Ok(Result) => Ok(Result),
					Err(Error) => {
						dev_log!(
							"scm",
							"warn: [SCM] vscode.diff sky://editor/diff did not answer ({:?}); returning null",
							Error
						);
						Ok(json!(null))
					},
				}
			})
		},

		_ => None,
	}
}
