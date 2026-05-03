#![allow(non_snake_case)]

//! Resolve the Tauri app-data prefix for THIS profile so logs
//! and aliasing pick the right `~/Library/Application Support/
//! land.editor.*.mountain` directory. The detection walks the
//! Application Support tree, prefers a strict suffix match
//! against the binary signature, falls back to the first
//! `*.mountain` candidate so a mismatch still produces a
//! usable path.

use std::sync::OnceLock;

static APP_DATA_PREFIX:OnceLock<Option<String>> = OnceLock::new();

pub fn Fn() -> &'static Option<String> { APP_DATA_PREFIX.get_or_init(DetectAppDataPrefix) }

fn BinarySignature() -> String {
	let PackageName = env!("CARGO_PKG_NAME");
	let Segments:Vec<&str> = PackageName.split('_').collect();
	let Take = Segments.len().min(4);
	let Start = Segments.len().saturating_sub(Take);
	Segments[Start..]
		.iter()
		.flat_map(|Segment| SplitPascalCaseIntoWords(Segment))
		.collect::<Vec<String>>()
		.join(".")
		.to_ascii_lowercase()
}

fn SplitPascalCaseIntoWords(Segment:&str) -> Vec<String> {
	let mut Words:Vec<String> = Vec::new();
	let mut Current = String::new();
	let mut PrevWasUpper = false;
	let mut PrevWasDigit = false;
	for Ch in Segment.chars() {
		let IsUpper = Ch.is_ascii_uppercase();
		let IsDigit = Ch.is_ascii_digit();
		let NeedBreak =
			!Current.is_empty() && ((IsUpper && !PrevWasUpper) || (IsDigit != PrevWasDigit && !Current.is_empty()));
		if NeedBreak {
			Words.push(std::mem::take(&mut Current));
		}
		Current.push(Ch);
		PrevWasUpper = IsUpper;
		PrevWasDigit = IsDigit;
	}
	if !Current.is_empty() {
		Words.push(Current);
	}
	Words.into_iter().filter(|Word| !Word.is_empty()).collect()
}

fn DetectAppDataPrefix() -> Option<String> {
	let Home = std::env::var("HOME").ok()?;
	let Base = format!("{}/Library/Application Support", Home);
	let Signature = BinarySignature();

	let mut FirstMatchingMountain:Option<String> = None;
	if let Ok(Entries) = std::fs::read_dir(&Base) {
		for Entry in Entries.flatten() {
			let Name = Entry.file_name();
			let Name = Name.to_string_lossy().into_owned();
			if !Name.starts_with("land.editor.") || !Name.contains("mountain") {
				continue;
			}
			if Name.ends_with(&Signature) {
				return Some(format!("{}/{}", Base, Name));
			}
			if FirstMatchingMountain.is_none() {
				FirstMatchingMountain = Some(format!("{}/{}", Base, Name));
			}
		}
	}
	FirstMatchingMountain
}
