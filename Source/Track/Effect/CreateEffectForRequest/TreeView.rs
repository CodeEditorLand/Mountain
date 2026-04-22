#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, TreeView::TreeViewProvider::TreeViewProvider};
use serde_json::{Value, json};
use tauri::Runtime;

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
						let Result = provider.RegisterTreeDataProvider(view_id, options).await;
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
