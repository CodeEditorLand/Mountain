#![allow(non_snake_case)]

//! Map a file extension to its IANA media type. Mirrors the inline helper in
//! `Binary/Build/Scheme.rs` so the cache layer is self-contained.

use std::path::Path;

pub fn Fn(Path:&Path) -> &'static str {
	match Path.extension().and_then(|S| S.to_str()).unwrap_or("") {
		"js" | "mjs" | "cjs" => "application/javascript; charset=utf-8",
		"css" => "text/css; charset=utf-8",
		"html" | "htm" => "text/html; charset=utf-8",
		"json" | "map" => "application/json; charset=utf-8",
		"svg" => "image/svg+xml",
		"png" => "image/png",
		"jpg" | "jpeg" => "image/jpeg",
		"gif" => "image/gif",
		"webp" => "image/webp",
		"woff" => "font/woff",
		"woff2" => "font/woff2",
		"ttf" => "font/ttf",
		"otf" => "font/otf",
		"wasm" => "application/wasm",
		"ico" => "image/x-icon",
		"txt" => "text/plain; charset=utf-8",
		"md" => "text/markdown; charset=utf-8",
		_ => "application/octet-stream",
	}
}
