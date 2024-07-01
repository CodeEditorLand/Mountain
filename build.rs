#![allow(non_snake_case)]

use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Toml {
	package: Package,
}

#[derive(Deserialize)]
struct Package {
	version: String,
}

fn main() {
	println!("cargo:rerun-if-changed=Cargo.toml");
	println!("cargo:rerun-if-changed=tauri.conf.json");

	let Version: Toml =
		toml::from_str(&fs::read_to_string("Cargo.toml").expect("Cannot Cargo.toml."))
			.expect("Cannot toml.");

	println!("{}", Version.package.version);

	tauri_build::build();

	// println!("cargo:rustc-env=CARGO_PKG_VERSION={}", Version);

	println!("WTF");

	// tauri_build::build()
}
