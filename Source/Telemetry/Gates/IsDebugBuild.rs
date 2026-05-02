#![allow(non_snake_case)]

//! Compile-time gate - evaluates to `true` under `cfg!(debug_assertions)`.

#[inline]
pub const fn Fn() -> bool { cfg!(debug_assertions) }
