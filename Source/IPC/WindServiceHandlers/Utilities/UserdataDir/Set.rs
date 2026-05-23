
//! Sets the userdata base directory (once, from Tauri's PathResolver).

pub fn Fn(Path:String) { let _ = super::BASE_DIR.set(Path); }
