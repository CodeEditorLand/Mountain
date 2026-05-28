use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	Track::Effect::{
		CreateEffectForRequest::Utilities::Params::{bool_at, i64_at, string_at, u64_at, u64_at_or, val_at},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$terminal:create" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();

				let options = val_at(&Parameters, 0);

				provider.CreateTerminal(options).await.map_err(|e| e.to_string())
			})
		},

		"$terminal:sendText" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();

				let terminal_id = i64_at(&Parameters, 0) as u64;

				let text = string_at(&Parameters, 1);

				provider
					.SendTextToTerminal(terminal_id, text)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"$terminal:dispose" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();

				let terminal_id = i64_at(&Parameters, 0) as u64;

				provider
					.DisposeTerminal(terminal_id)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"Terminal.Resize" | "$terminal:resize" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();

				let terminal_id = match Parameters.get(0) {
					Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
					Some(Value::String(s)) => {
						s.rsplit(':').next().and_then(|token| token.parse::<u64>().ok()).unwrap_or(0)
					},
					_ => 0,
				};

				let cols = u64_at_or(&Parameters, 1, 80) as u16;

				let rows = u64_at_or(&Parameters, 2, 24) as u16;

				provider
					.ResizeTerminal(terminal_id, cols, rows)
					.await
					.map(|()| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"Terminal.GetProcessId" => {
			crate::effect!(run_time, {
				let Provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();

				let Handle = val_at(&Parameters, 0);

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
		},

		// `$terminal:show` / `$terminal:hide` - Cocoon's window namespace shim
		// calls these when extensions invoke `terminal.show(preserveFocus)` or
		// `terminal.hide()`. Wire through the same `ShowTerminal` / `HideTerminal`
		// provider path that the `terminal:show` / `terminal:hide` IPC handlers
		// use so both call sites share one implementation.
		"$terminal:show" | "Terminal.Show" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();

				let terminal_id = u64_at(&Parameters, 0);

				let preserve_focus = bool_at(&Parameters, 1);

				provider
					.ShowTerminal(terminal_id, preserve_focus)
					.await
					.map(|()| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"$terminal:hide" | "Terminal.Hide" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();

				let terminal_id = u64_at(&Parameters, 0);

				provider
					.HideTerminal(terminal_id)
					.await
					.map(|()| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		_ => None,
	}
}
