// File: Rpc/Args/Secrets/SetSecretArgument.rs

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SetSecretArgument {
	// Identifier of the extension storing the secret.
	// Used to namespace secrets in the keyring.
	#[serde(alias = "extensionId")]
	pub ExtensionIdentifier:String,
	// The specific key under which to store the secret.
	pub Key:String,
	// The secret value to store.
	pub Value:String,
}
