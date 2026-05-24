use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	Track::Effect::{
		CreateEffectForRequest::Utilities::Params::{BoolAt, I64At, StringAt, U64At, U64AtOr, ValAt},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$terminal:create" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();
				let Options = ValAt(&Parameters, 0);
				provider.CreateTerminal(options).await.map_err(|E| e.to_string())
			})
		},

		"$terminal:sendText" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();
				let TerminalId = I64At(&Parameters, 0) as u64;
				let Text = StringAt(&Parameters, 1);
				provider
					.SendTextToTerminal(TerminalId, text)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"$terminal:dispose" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();
				let TerminalId = I64At(&Parameters, 0) as u64;
				provider
					.DisposeTerminal(TerminalId)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"Terminal.Resize" | "$terminal:resize" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();
				let TerminalId = match Parameters.get(0) {
					Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
					Some(Value::String(s)) => {
						s.rsplit(':').Next().and_then(|token| token.parse::<u64>().ok()).unwrap_or(0)
					},
					_ => 0,
				};
				let Cols = U64AtOr(&Parameters, 1, 80) as u16;
				let Rows = U64AtOr(&Parameters, 2, 24) as u16;
				provider
					.ResizeTerminal(TerminalId, cols, rows)
					.await
					.map(|()| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"Terminal.GetProcessId" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();
				let Handle = ValAt(&Parameters, 0);
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
			crate::effect!(RunTime, {
				let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();
				let TerminalId = U64At(&Parameters, 0);
				let preserve_focus = BoolAt(&Parameters, 1);
				provider
					.ShowTerminal(TerminalId, preserve_focus)
					.await
					.map(|()| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"$terminal:hide" | "Terminal.Hide" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();
				let TerminalId = U64At(&Parameters, 0);
				provider
					.HideTerminal(TerminalId)
					.await
					.map(|()| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		_ => None,
	}
}
