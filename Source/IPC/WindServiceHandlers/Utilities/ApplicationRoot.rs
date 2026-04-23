#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! `/Static/Application/` → Sky Target real path. The static root is seeded
//! once by `AppLifecycle::Dirs` with the resolved `Sky/Target` directory
//! (debug) or the bundle resource dir (release) so `file:read` on any
//! `Static/Application/...` URI lands on the real asset.

/// The real filesystem root for `/Static/Application/` paths.
/// Set once at startup with the Sky Target directory.
static STATIC_APPLICATION_ROOT:std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn set_static_application_root(Path:String) { let _ = STATIC_APPLICATION_ROOT.set(Path); }

pub fn get_static_application_root() -> Option<String> { STATIC_APPLICATION_ROOT.get().cloned() }
