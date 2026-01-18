#![allow(non_snake_case)]
use std::fs::read_to_string;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
struct Toml {
	package:Package,
}

#[derive(Deserialize)]
struct Package {
	version:String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	if !tauri_build::is_dev() {
		println!("cargo:rerun-if-changed=Cargo.toml");

		println!("cargo:rerun-if-changed=tauri.conf.json");

		println!("cargo:rerun-if-changed=tauri.conf.json5");

		let Version = toml::from_str::<Toml>(&read_to_string("Cargo.toml")?)?.package.version;

		let File = if std::path::Path::new("tauri.conf.json5").exists() {
			"tauri.conf.json5"
		} else {
			"tauri.conf.json"
		};

		let Content = read_to_string(File)?;

		let mut Tauri:Value = match json5::from_str(&Content) {
			Ok(Value) => Value,
			Err(_) => serde_json::from_str(&Content)?,
		};

		Tauri.get_mut("version").map(|Entry| *Entry = Value::String(Version.clone()));

		let mut Serializer =
			serde_json::Serializer::with_formatter(Vec::new(), serde_json::ser::PrettyFormatter::with_indent(b"\t"));

		Tauri.serialize(&mut Serializer)?;

		std::fs::write(File, String::from_utf8(Serializer.into_inner())?)?;

		println!("cargo:rustc-env=CARGO_PKG_VERSION={}", Version);
	}

	println!("cargo:rerun-if-changed=Proto/Vine.proto");

	tonic_prost_build::configure()
		.build_server(true)
		.build_client(true)
		.out_dir("Source/Vine/Generated")
		.compile_well_known_types(true)
		.compile_protos(&["Proto/Vine.proto"], &["Proto"])?;

	tauri_build::build();

	Ok(())
}
