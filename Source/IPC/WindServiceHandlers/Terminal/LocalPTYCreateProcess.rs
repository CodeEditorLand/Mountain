#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `localPty:createProcess`.
//! VS Code's `IPtyService.createProcess` is typed `Promise<number>`.
//! The workbench does `new LocalPty(id, …)` and keys `_ptys` by that integer;
//! returning the full `{ id, name, pid }` object causes every subsequent
//! `_ptys.get(<integer>)` lookup to return `undefined` and xterm to receive
//! zero bytes. This handler strips down to the integer id.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Terminal::TerminalCreate::Fn as TerminalCreate,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	match TerminalCreate(RunTime, Arguments).await {
		Ok(Response) => {
			let TerminalIdOption = Response.get("id").and_then(serde_json::Value::as_u64);

			match TerminalIdOption {
				Some(TerminalId) if TerminalId > 0 => Ok(serde_json::json!(TerminalId)),

				Some(_) | None => {
					// Defensive: if `CreateTerminal` returned without a usable id
					// (shape drift or `GetNextTerminalIdentifier` regression),
					// surface an error so the workbench binds `LocalPty(0, …)`
					// and every subsequent `_proxy.input(0, data)` fails loudly.
					crate::dev_log!(
						"terminal",
						"error: [localPty:createProcess] CreateTerminal returned no usable id; response={:?}",
						Response
					);

					Err(format!(
						"localPty:createProcess: CreateTerminal returned no terminal id (response={})",
						Response
					))
				},
			}
		},

		Err(Error) => Err(Error),
	}
}
