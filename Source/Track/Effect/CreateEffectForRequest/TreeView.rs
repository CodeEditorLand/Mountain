#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, IPC::SkyEvent::SkyEvent, TreeView::TreeViewProvider::TreeViewProvider};
use serde_json::{Value, json};
use tauri::{Emitter, Runtime};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(
	MethodName:&str,
	Parameters:Value,
) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$tree:register" | "tree.register" => {
			let DispatchEnterNs = std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|D| D.as_nanos())
				.unwrap_or(0);
			dev_log!(
				"grpc",
				"[LandFix:Tree] dispatch-enter method={} t_ns={}",
				MethodName,
				DispatchEnterNs
			);

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let DispatchAt = std::time::Instant::now();
						let BodyStartNs = std::time::SystemTime::now()
							.duration_since(std::time::UNIX_EPOCH)
							.map(|D| D.as_nanos())
							.unwrap_or(0);
						let provider:Arc<dyn TreeViewProvider> = run_time.Environment.Require();
						let first = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let (view_id, options) = if Parameters.get(2).is_some() {
							let vid =
								Parameters.get(1).and_then(Value::as_str).unwrap_or(first).to_string();
							let opts = Parameters.get(2).cloned().unwrap_or_default();
							(vid, opts)
						} else {
							let vid = first.to_string();
							let opts = Parameters.get(1).cloned().unwrap_or_default();
							(vid, opts)
						};
						let ViewIdForLog = view_id.clone();
						dev_log!(
							"grpc",
							"[LandFix:Tree] body-start view={} t_ns={}",
							ViewIdForLog,
							BodyStartNs
						);
						let Result = provider.RegisterTreeDataProvider(view_id.clone(), options.clone()).await;
						let RegisteredNs = std::time::SystemTime::now()
							.duration_since(std::time::UNIX_EPOCH)
							.map(|D| D.as_nanos())
							.unwrap_or(0);
						dev_log!(
							"grpc",
							"[LandFix:Tree] registered view={} elapsed={}ms t_ns={}",
							ViewIdForLog,
							DispatchAt.elapsed().as_millis(),
							RegisteredNs
						);
						dev_log!(
							"tree-view",
							"[TreeView] register view={} result={} elapsed={}ms",
							ViewIdForLog,
							if Result.is_ok() { "ok" } else { "err" },
							DispatchAt.elapsed().as_millis()
						);

						// Notify Wind/Sky that a data provider now exists for this
						// view, so the renderer can set `treeView.dataProvider` on
						// the matching ITreeView instance and replace the default
						// "no data provider registered" message. Without this
						// emit, `vs/workbench/browser/parts/views/treeView.ts`
						// keeps `_dataProvider === undefined` and every extension
						// tree view stays empty (GitLens, debug, SCM, tasks, etc.).
						if Result.is_ok() {
							if let Err(Error) = run_time.Environment.ApplicationHandle.emit(
								SkyEvent::TreeViewCreate.AsStr(),
								json!({
									"viewId": view_id,
									"options": options,
								}),
							) {
								dev_log!(
									"grpc",
									"warn: [LandFix:Tree] failed to emit {} for view={}: {}",
									SkyEvent::TreeViewCreate.AsStr(),
									ViewIdForLog,
									Error
								);
								dev_log!(
									"tree-view",
									"[TreeView] emit-fail channel={} view={} error={}",
									SkyEvent::TreeViewCreate.AsStr(),
									ViewIdForLog,
									Error
								);
							} else {
								dev_log!(
									"tree-view",
									"[TreeView] emit-ok channel={} view={}",
									SkyEvent::TreeViewCreate.AsStr(),
									ViewIdForLog
								);
							}
						}

						Result.map(|_| json!(null)).map_err(|e| e.to_string())
					})
				};
			Some(Ok(Box::new(effect)))
		},

		"tree.unregister" | "tree.dispose" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let handle = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						dev_log!("ipc", "[tree.unregister] handle={}", handle);
						Ok(json!(null))
					})
				};
			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
