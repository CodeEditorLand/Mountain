#![allow(non_snake_case)]

//! Hash a `viewId` string into the same `u32` that
//! `RegisterTreeViewProvider::Fn` uses as a registration handle. Lets
//! `GetTreeChildren::Fn` look up the registered provider without the
//! caller passing the handle through the wire.

pub fn Fn(ViewIdentifier:&str) -> u32 {

	ViewIdentifier
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32))
}
