//! `JsonValueHelpers::ReqString`

use serde_json::Value;

pub fn Fn(Args:&[Value], N:usize, Msg:&str) -> Result<String, String> {
	ReqStr(Args, N, Msg).map(str::to_string)
}
