#![allow(unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider,
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$scm:createSourceControl" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let resource = Parameters.get(0).cloned().unwrap_or_default();
						provider
							.CreateSourceControl(resource)
							.await
							.map(|handle| json!(handle))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$scm:updateSourceControl" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let handle = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(0);
						let update = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.UpdateSourceControl(handle, update)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$scm:updateGroup" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let handle = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(0);
						let group_data = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.UpdateSourceControlGroup(handle, group_data)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$scm:registerInputBox" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let handle = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(0);
						let options = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.RegisterInputBox(handle, options)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
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
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						dev_log!(
							"scm",
							"[SCM] vscode.diff forwarding to sky://editor/diff params={:?}",
							Parameters
						);

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
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
