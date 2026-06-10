//! Cocoon → Mountain `terminal.envCollection.<op>` notifications.
//! Each `op` (`replace` / `append` / `prepend` / `delete` / `clear` /
//! `setPersistent` / `setDescription`) mutates the in-memory env
//! collection registry keyed by extension id. Mutations are picked up
//! by every PTY spawn that runs after the notification arrives - no
//! retro-active mutation of running terminals (matches VS Code
//! semantics).

use serde_json::Value;

use crate::{
	Environment::TerminalEnvCollection,
	Vine::Server::MountainVinegRPCService::MountainVinegRPCService,
	dev_log,
};

pub async fn TerminalEnvCollectionDispatch(_Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	let Suffix = MethodName.strip_prefix("terminal.envCollection.").unwrap_or(MethodName);

	let (ExtensionId, Variable, ValueStr) = TerminalEnvCollection::ParsePayload(Parameter);

	if ExtensionId.is_empty() && Suffix != "clear" && Suffix != "setPersistent" && Suffix != "setDescription" {
		// Extension id is mandatory for every per-variable op. Without
		// it we'd land mutations in a global "" bucket that no
		// extension can ever clear; refuse rather than corrupt state.
		dev_log!(
			"terminal",
			"warn: [EnvCollection] {} called without extensionId - dropped",
			Suffix
		);

		return;
	}

	match Suffix {
		"replace" => TerminalEnvCollection::Replace(&ExtensionId, Variable, ValueStr),

		"append" => TerminalEnvCollection::Append(&ExtensionId, Variable, ValueStr),

		"prepend" => TerminalEnvCollection::Prepend(&ExtensionId, Variable, ValueStr),

		"delete" => TerminalEnvCollection::Delete(&ExtensionId, &Variable),

		"clear" => TerminalEnvCollection::Clear(&ExtensionId),

		"setPersistent" => {
			let Persistent = Parameter.get("persistent").and_then(|V| V.as_bool()).unwrap_or(false);

			TerminalEnvCollection::SetPersistent(&ExtensionId, Persistent);
		},

		"setDescription" => {
			let Description = Parameter.get("description").and_then(|V| V.as_str()).map(|S| S.to_string());

			TerminalEnvCollection::SetDescription(&ExtensionId, Description);
		},

		Other => {
			dev_log!("terminal", "warn: [EnvCollection] unknown op '{}'", Other);
		},
	}
}
