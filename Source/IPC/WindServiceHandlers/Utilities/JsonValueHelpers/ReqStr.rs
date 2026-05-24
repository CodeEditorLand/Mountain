//! `JsonValueHelpers::ReqStr`

use serde_json::Value;

pub fn Fn<'a>(Args:&'a [Value], N:usize, Msg:&str) -> Result<&'a str, String> {
	Args.get(N).and_then(Value::as_str).ok_or_else(|| Msg.to_string())
}
