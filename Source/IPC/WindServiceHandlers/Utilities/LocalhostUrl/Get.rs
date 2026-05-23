#![allow(non_snake_case)]

//! Returns the cached localhost plugin base URL.

pub fn Fn() -> Option<String> { super::URL.get().cloned() }
