pub fn Matches(MethodName:&str) -> bool {
	MethodName == "vscode.diff" || MethodName == "$scm:openDiff" || MethodName.starts_with("$scm:")
}

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider,
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	Track::Effect::{
		CreateEffectForRequest::Utilities::Params::{i64_at, val_at},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	dev_log!("scm", "[SCM] CreateEffect method={}", MethodName);

	match MethodName {
		"$scm:createSourceControl" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();

				let resource = val_at(&Parameters, 0);

				provider
					.CreateSourceControl(resource)
					.await
					.map(|handle| json!(handle))
					.map_err(|e| e.to_string())
			})
		},

		"$scm:updateSourceControl" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();

				let handle = i64_at(&Parameters, 0) as u32;

				let update = val_at(&Parameters, 1);

				provider
					.UpdateSourceControl(handle, update)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"$scm:updateGroup" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();

				let handle = i64_at(&Parameters, 0) as u32;

				let group_data = val_at(&Parameters, 1);

				provider
					.UpdateSourceControlGroup(handle, group_data)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"$scm:registerInputBox" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();

				let handle = i64_at(&Parameters, 0) as u32;

				let options = val_at(&Parameters, 1);

				provider
					.RegisterInputBox(handle, options)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
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
		//
		// `git.openChange` / `git.openFile` are the actual command IDs
		// the built-in vscode.git extension uses as the `.command` field
		// on its resource state entries - clicking a changed file in the
		// SCM sidebar dispatches one of these. We alias them onto the
		// same diff-forward path so the diff editor opens.
		"vscode.diff" | "$scm:openDiff" | "git.openChange" | "git.openFile" => {
			crate::effect!(run_time, {
				dev_log!("scm", "[SCM] diff forwarding to sky://editor/diff params={:?}", Parameters);

				match crate::Environment::UserInterfaceProvider::SendUserInterfaceRequest(
					&run_time.Environment,
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
