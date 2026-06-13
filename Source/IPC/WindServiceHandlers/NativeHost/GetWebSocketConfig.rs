//! Wire method: `nativeHost:getWebSocketConfig`.
//!
//! Returns Cocoon WS + Mountain Mist WS ports/secrets so the renderer
//! can establish direct WebSocket transports.

use serde_json::{Value, json};

pub async fn Fn() -> Result<Value, String> {
	use crate::ProcessManagement::CocoonManagement::{WsPort, WsSecretHex};

	let MountainPort = std::env::var("MountainWebSocketPort")
		.ok()
		.and_then(|P| P.parse::<u16>().ok())
		.unwrap_or(0);

	let MountainSecret = std::env::var("MountainWebSocketSecret").ok().unwrap_or_default();

	Ok(json!({
		"port": WsPort(),
		"secret": WsSecretHex(),
		"mountainPort": MountainPort,
		"mountainSecret": MountainSecret,
	}))
}
