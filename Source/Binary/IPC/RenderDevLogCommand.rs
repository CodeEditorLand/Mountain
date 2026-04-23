//! # RenderDevLogCommand
//!
//! Bridges TypeScript-side tagged logs (Wind / Sky / Cocoon running in the
//! WebView) into Mountain's `dev_log!` file sink so a single
//! `Mountain.dev.log` carries the complete tag-filterable picture of a
//! session, not just the Rust portion.
//!
//! ## Why
//!
//! Wind's `Function/DevLog.ts` prints to `console.log`, visible only in
//! DevTools. Mountain's `dev_log!` writes to a timestamped
//! `Mountain.dev.log` with `LAND_DEV_LOG` tag filtering. The two observe
//! the same boot but the outputs live in two places - reviewing a session
//! means cross-referencing browser console + tail-f on the file. This
//! command lets any TS caller mirror a tagged line into the file sink so
//! `LAND_DEV_LOG=channel-stub,git tail -f Mountain.dev.log` captures both
//! sides filtered identically.
//!
//! ## Contract
//!
//! - `Tag` is matched against `LAND_DEV_LOG` exactly as Rust tags are;
//!   disabled tags produce no output (cheap no-op on the Mountain side).
//! - `Message` is the fully-formatted log string - the caller has
//!   already done any interpolation.
//! - The command is fire-and-forget from the renderer's perspective
//!   (returns `()` immediately); failures to write are swallowed.
//! - Prefix `[RenderDevLog]` is added so grep can always separate
//!   TS-originated lines from native `dev_log!` entries that share the
//!   same tag.

#![allow(non_snake_case)]

use crate::dev_log;

#[tauri::command]
pub fn RenderDevLog(Tag:String, Message:String) {
	// The `dev_log!` macro expands to a conditional on `IsEnabled(Tag)`
	// - when the tag is off, the entire call is a no-op besides the
	//   string allocation on the caller side. `$Tag:expr` in the macro
	//   accepts `&str` / `&String` / `String` interchangeably.
	let TagRef:&str = &Tag;
	dev_log!(TagRef, "[RenderDevLog] {}", Message);
}
