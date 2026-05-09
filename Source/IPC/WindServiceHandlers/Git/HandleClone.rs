#![allow(non_snake_case)]

//! `localGit:clone(operationId, cloneUrl, targetPath, ref?)`.
//! Optional `ref` becomes `--branch <ref>` so callers can
//! shallow-clone a tag or branch.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Git::Shared::RunGit;

pub async fn HandleClone(Arguments:Vec<Value>) -> Result<Value, String> {

	let OperationId = Arguments.first().and_then(Value::as_str).unwrap_or("").to_string();

	let CloneURL = Arguments.get(1).and_then(Value::as_str).unwrap_or("").to_string();

	let TargetPath = Arguments.get(2).and_then(Value::as_str).unwrap_or("").to_string();

	let Reference = Arguments.get(3).and_then(Value::as_str).map(str::to_string);

	if CloneURL.is_empty() || TargetPath.is_empty() {

		return Err("git:clone requires cloneUrl and targetPath".to_string());
	}

	let mut Argv:Vec<String> = vec!["clone".to_string()];

	if let Some(Ref) = Reference {

		Argv.push("--branch".to_string());

		Argv.push(Ref);
	}

	Argv.push("--".to_string());

	Argv.push(CloneURL);

	Argv.push(TargetPath);

	let (ExitCode, _, Stderr) = RunGit(&OperationId, &Argv, None).await?;

	if ExitCode != 0 {

		return Err(format!("git clone failed: {}", Stderr));
	}

	Ok(Value::Null)
}
