#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! `/Static/Application/` → Sky Target real path.
//! State held here; `Get` and `Set` expose atomic accessors.

pub(crate) static ROOT:std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub mod Get;

pub mod Set;
