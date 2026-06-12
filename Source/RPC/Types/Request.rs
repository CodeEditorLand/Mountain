//! Generic request envelope for shared RPC types.
/// Typed request envelope wrapping a single data payload.
pub struct Struct<T> {
	pub data:T,
}
