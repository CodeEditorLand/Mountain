#![allow(non_snake_case)]

//! Generic request envelope for shared RPC types.

pub struct Struct<T> {
	pub data:T,
}
