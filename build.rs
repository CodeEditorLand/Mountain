#![allow(non_snake_case)]

use serde::Deserialize;
use serde_json::Value;
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
	if !tauri_build::is_dev() {
		println!("cargo:rerun-if-changed=Cargo.toml");
		println!("cargo:rerun-if-changed=tauri.conf.json");

		let Cargo: Toml =
			toml::from_str(&fs::read_to_string("Cargo.toml").expect("Cannot Cargo.toml."))
				.expect("Cannot toml.");

		let Version = Cargo.package.version;

		let mut Tauri: Value = serde_json::from_str(
			&fs::read_to_string("tauri.conf.json").expect("Cannot tauri.conf.json."),
		)
		.expect("Cannot JSON.");

		Tauri.get_mut("version").map(|Entry| *Entry = Value::String(Version.clone()));

		fs::write("tauri.conf.json", serde_json::to_string_pretty(&Tauri).expect("Cannot JSON."))
			.expect("Cannot tauri.conf.json.");

		println!("cargo:rustc-env=CARGO_PKG_VERSION={}", Version);
	}

	tauri_build::build();
}
