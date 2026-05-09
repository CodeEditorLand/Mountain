#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$terminal:create" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
						let options = Parameters.get(0).cloned().unwrap_or_default();
						provider.CreateTerminal(options).await.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$terminal:sendText" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
						let terminal_id = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u64).unwrap_or(0);
						let text = Parameters.get(1).and_then(Value::as_str).unwrap_or("").to_string();
						provider
							.SendTextToTerminal(terminal_id, text)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$terminal:dispose" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
						let terminal_id = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u64).unwrap_or(0);
						provider
							.DisposeTerminal(terminal_id)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"Terminal.Resize" | "$terminal:resize" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
						let terminal_id = match Parameters.get(0) {
							Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
							Some(Value::String(s)) => {
								s.rsplit(':').next().and_then(|token| token.parse::<u64>().ok()).unwrap_or(0)
							},
							_ => 0,
						};
						let cols = Parameters.get(1).and_then(Value::as_u64).map(|n| n as u16).unwrap_or(80);
						let rows = Parameters.get(2).and_then(Value::as_u64).map(|n| n as u16).unwrap_or(24);
						provider
							.ResizeTerminal(terminal_id, cols, rows)
							.await
							.map(|()| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"Terminal.GetProcessId" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use CommonLibrary::{
							Environment::Requires::Requires,
							Terminal::TerminalProvider::TerminalProvider,
						};
						let Provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
						let Handle = Parameters.get(0).cloned().unwrap_or_default();
						let Id:u64 = if let Some(n) = Handle.as_u64() {
							n
						} else if let Some(s) = Handle.as_str() {
							s.trim_start_matches("terminal:").parse().unwrap_or(0)
						} else {
							0
						};
						match Provider.GetTerminalProcessId(Id).await {
							Ok(Some(Pid)) => Ok(json!(Pid)),
							Ok(None) => Ok(Value::Null),
							Err(Error) => Err(Error.to_string()),
						}
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
