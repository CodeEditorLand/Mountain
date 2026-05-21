#![allow(non_snake_case)]

//! Resolve the Tauri app-data prefix for THIS profile so logs
//! and aliasing pick the right `~/Library/Application Support/
//! land.editor.*.mountain` directory. The detection walks the
//! Application Support tree, prefers a strict suffix match
//! against the binary signature, falls back to the first
//! `*.mountain` candidate so a mismatch still produces a
//! usable path.

use std::sync::{Mutex, OnceLock};

// Two-phase resolution:
// • `RESOLVED` is set permanently once we find a real prefix.
// • `FAILED_ONCE` records whether the first attempt returned None so
//   subsequent writes can retry after Tauri has created the directory,
//   rather than caching None forever and routing all logs to /tmp.
static RESOLVED:OnceLock<String> = OnceLock::new();

static RETRY:Mutex<bool> = Mutex::new(true);

pub fn Fn() -> Option<&'static str> {
	if let Some(S) = RESOLVED.get() {
		return Some(S.as_str());
	}

	// Fast-path: guard without taking the mutex if we already have a result.
	let Ok(mut Guard) = RETRY.try_lock() else {
		return None;
	};

	if !*Guard {
		return None;
	}

	if let Some(Prefix) = DetectAppDataPrefix() {
		// RESOLVED may already be set by a concurrent caller - that's fine.
		let _ = RESOLVED.set(Prefix);

		*Guard = false;
		return RESOLVED.get().map(String::as_str);
	}

	// Not found yet - leave RETRY=true so the next write retries.
	None
}

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
