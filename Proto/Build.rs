// ORIGIN INFORMATION:
// This code block was extracted by a script.
// Source Markdown File: Backup/MountainFinalSync/Document/14_MODEL.md
// Source Block Index in MD (Overall): 3
// Original Fence Info String: (empty)
// Content SHA256 (of this block):
// 7bbaddc3d4a89cda326f09f7335e4a06b4cc0179bb7e5601667744c7198028e1 Extracted to
// File: Backup/MountainFinalSync/Code/mountain/build.rs Extraction Timestamp:
// 2025-06-18T18:53:42.858Z --- END OF ORIGIN INFORMATION ---

// @file build.rs
// @brief The build script for the Mountain crate.
// @description This script uses `tonic-build` to compile the Protobuf
// definitions in `Vine.proto` into Rust code, which is then included by the
// `Vine::Generated` module.

fn main() -> Result<(), Box<dyn std::error::Error>> {
	println!("cargo:rerun-if-changed=Vine.proto");

	tonic_build::configure()
		.build_server(true)
		.build_client(true)
		.out_dir("Source/Vine/Generated")
		.compile(&["Vine.proto"], &["proto"])?;

	Ok(())
}
