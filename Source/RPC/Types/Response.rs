#![allow(non_snake_case)]

//! Generic response envelope for shared RPC types.

pub struct Struct<T> {
	pub data:T,
}
