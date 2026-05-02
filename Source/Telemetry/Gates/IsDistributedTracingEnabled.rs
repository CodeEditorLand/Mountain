#![allow(non_snake_case)]

//! Compile-time gate - `true` under the `DistributedTracing` feature.

#[inline]
pub const fn Fn() -> bool { cfg!(feature = "DistributedTracing") }
