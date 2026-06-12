//! Generic response envelope for shared RPC types.
/// Typed response envelope wrapping a single data payload.
pub struct Struct<T> {
	pub data:T,
}
