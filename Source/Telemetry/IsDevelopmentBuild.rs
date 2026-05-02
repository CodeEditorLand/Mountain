#![allow(non_snake_case)]

//! `true` for non-release builds OR when compiled with `--features
//! Development`. Used to gate verbose log output and developer-only menus.

#[inline]
pub const fn Fn() -> bool { cfg!(feature = "Development") || cfg!(debug_assertions) }
