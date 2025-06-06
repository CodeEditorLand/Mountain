// File: Rpc/Args/Secrets/GetSecretArgument.rs

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct GetSecretArgument {
	// Identifier of the extension requesting the secret.
	// Used to namespace secrets in the keyring.
	#[serde(alias = "extensionId")]
	pub ExtensionIdentifier:String,
	// The specific key for the secret to retrieve.
	pub Key:String,
}
