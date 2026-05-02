#![allow(non_snake_case)]

//! Capture gate. Combines the compile-time `debug_assertions` check with
//! the `Report` env-var switch baked at build time. Cheap early-exit
//! consulted by every capture path.

use crate::Binary::Build::PostHogPlugin::Constants;

pub fn Fn() -> bool {
	if !cfg!(debug_assertions) {
		return false;
	}
	!matches!(Constants::POSTHOG_ENABLED, "false" | "0" | "off")
}
