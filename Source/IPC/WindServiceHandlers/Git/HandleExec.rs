//! `localGit:exec` - arbitrary `git` argv. Used by the Git
//! extension for commands not on the curated `clone/pull/…`
//! list. Accepts both the modern `{ Arguments, cwd?,
//! operationId? }` shape and the legacy positional
//! `(argv: string[], cwd?: string)`.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::{
	Git::Shared::{AsStringArray::Fn as AsStringArray, RunGit::Fn as RunGit},
	Utilities::JsonValueHelpers::arg_string,
};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let (Argv, Cwd, OperationId) = match Arguments.first() {
		Some(First) if First.is_object() => {
			let Obj = First.as_object().unwrap();

			let Argv = Obj.get("Arguments").map(AsStringArray).unwrap_or_default();

			let Cwd = Obj.get("cwd").and_then(Value::as_str).unwrap_or("").to_string();

			let OperationId = Obj.get("operationId").and_then(Value::as_str).unwrap_or("").to_string();

			(Argv, Cwd, OperationId)
		},

		Some(First) if First.is_array() => {
			let Argv = AsStringArray(First);

			let Cwd = arg_string(&Arguments, 1);

			(Argv, Cwd, String::new())
		},

		_ => (Vec::new(), String::new(), String::new()),
	};

	if Argv.is_empty() {
		return Err("git:exec requires non-empty Arguments".to_string());
	}

	let OperationIdRef = if OperationId.is_empty() {
		uuid::Uuid::new_v4().to_string()
	} else {
		OperationId
	};

	let CwdOpt = if Cwd.is_empty() { None } else { Some(Cwd.as_str()) };

	let (ExitCode, Stdout, Stderr) = RunGit(&OperationIdRef, &Argv, CwdOpt).await?;

	Ok(json!({
		"stdout": Stdout,
		"stderr": Stderr,
		"exitCode": ExitCode,
	}))
}
