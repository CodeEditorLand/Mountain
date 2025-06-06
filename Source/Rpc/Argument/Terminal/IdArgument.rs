

use serde::Deserialize;

// This DTO is used for RPC calls that primarily target a terminal
// using its unique identifier, such as Hide or Dispose operations.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct IdArgument {
	// The unique numeric identifier of the terminal instance.
	pub Id:u64,
}
