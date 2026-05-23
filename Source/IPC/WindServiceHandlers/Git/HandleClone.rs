//! `localGit:clone(operationId, cloneUrl, targetPath, ref?)`.
//! Optional `ref` becomes `--branch <ref>` so callers can
//! shallow-clone a tag or branch.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::{Git::Shared::RunGit::Fn as RunGit, Utilities::JsonValueHelpers::arg_string};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let OperationId = arg_string(&Arguments, 0);

	let CloneURL = arg_string(&Arguments, 1);

	let TargetPath = arg_string(&Arguments, 2);

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
