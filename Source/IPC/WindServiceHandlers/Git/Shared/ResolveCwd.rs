#![allow(non_snake_case)]

//! Resolves a raw cwd string to a PathBuf. Falls back to current dir.

use std::path::PathBuf;

pub fn Fn(Raw:&str) -> PathBuf {
	if Raw.is_empty() {
		std::env::current_dir().unwrap_or_default()
	} else {
		PathBuf::from(Raw)
	}
}
