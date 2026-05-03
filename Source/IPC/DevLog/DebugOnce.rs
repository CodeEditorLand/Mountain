#![allow(non_snake_case)]

//! Emit a tagged dev-log line exactly once per process, keyed
//! on `Key`. Subsequent calls with the same key are dropped
//! from the console; the file sink still records the first
//! occurrence so post-mortems show every probe path that
//! fired.

use std::{
	collections::HashSet,
	sync::{Mutex, OnceLock},
};

use crate::IPC::DevLog::{IsEnabled, WriteToFile};

static DEBUG_ONCE_KEYS:OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn DebugOnceKeys() -> &'static Mutex<HashSet<String>> { DEBUG_ONCE_KEYS.get_or_init(|| Mutex::new(HashSet::new())) }

pub fn Fn(Tag:&str, Key:&str, Line:&str) {
	if let Ok(mut Keys) = DebugOnceKeys().lock() {
		if !Keys.insert(Key.to_string()) {
			return;
		}
	}
	if IsEnabled::Fn(Tag) || IsEnabled::Fn("all") {
		let Formatted = format!("[DEV:{}] {}", Tag.to_uppercase(), Line);
		eprintln!("{}", Formatted);
		WriteToFile::Fn(&Formatted);
	} else {
		let Formatted = format!("[DEV:{}/once] {}", Tag.to_uppercase(), Line);
		WriteToFile::Fn(&Formatted);
	}
}
