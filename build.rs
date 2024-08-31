#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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

		let Version =
			toml::from_str::<Toml>(&fs::read_to_string("Cargo.toml").expect("Cannot Cargo.toml."))
				.expect("Cannot toml.")
				.package
				.version;

		let mut Tauri: Value = serde_json::from_str(
			&fs::read_to_string("tauri.conf.json").expect("Cannot tauri.conf.json."),
		)
		.expect("Cannot JSON.");

		Tauri.get_mut("version").map(|Entry| *Entry = Value::String(Version.clone()));

		let Formatter = serde_json::ser::PrettyFormatter::with_indent(b"	");

		let Buffer = Vec::new();
		let mut Serializer = serde_json::Serializer::with_formatter(Buffer, Formatter);

		Tauri.serialize(&mut Serializer).expect("Cannot serialize.");

		fs::write("tauri.conf.json", Tauri.to_string()).expect("Cannot tauri.conf.json.");

		println!("cargo:rustc-env=CARGO_PKG_VERSION={}", Version);
	}

	tauri_build::build();
}
