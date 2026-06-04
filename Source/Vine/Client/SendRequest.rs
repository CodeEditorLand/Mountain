//! Send a request and await a response. Validates method-name length
//! and message size, prefers the streaming multiplexer when
//! `LAND_VINE_STREAMING=1` is on (falls through to unary on any failure
//! except the authoritative streaming-path timeout), enforces a per-call
//! timeout via `tokio::time::timeout`, and updates per-connection
//! activity / failure metadata on completion.

use serde_json::Value;

use crate::Vine::Error::VineError;

pub async fn Fn(
	SideCarIdentifier:&str,

	Method:String,

	Parameters:Value,

	TimeoutMilliseconds:u64,
) -> Result<Value, VineError> {
	::Vine::Client::SendRequest::Fn(SideCarIdentifier, Method, Parameters, TimeoutMilliseconds).await
}
