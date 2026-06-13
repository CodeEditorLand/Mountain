//! Wire method: `nativeHost:isPortFree`.
//!
//! Checks whether a TCP port is free by attempting to bind on localhost.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_u64;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Port = arg_u64(&Arguments, 0) as u16;

	if Port == 0 {
		Ok(json!(false))
	} else {
		let Free = tokio::net::TcpListener::bind(std::net::SocketAddr::from(([127, 0, 0, 1], Port)))
			.await
			.is_ok();

		Ok(json!(Free))
	}
}
