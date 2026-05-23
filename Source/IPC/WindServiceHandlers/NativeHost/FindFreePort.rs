
//! Wire method: `nativeHost:findFreePort`.
//! Scans 100 ports starting from `Arguments[0]` (default 9000) and returns the
//! first free one. Returns 0 when nothing is free in-range so callers can
//! distinguish "search exhausted" from a genuine port 0.

use serde_json::{Value, json};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let StartPort = Arguments.get(0).and_then(|V| V.as_u64()).unwrap_or(9000) as u16;

	for Port in StartPort..StartPort + 100 {
		if std::net::TcpListener::bind(("127.0.0.1", Port)).is_ok() {
			return Ok(json!(Port));
		}
	}

	Ok(json!(0))
}
