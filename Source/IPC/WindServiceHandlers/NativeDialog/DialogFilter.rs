//! Tauri-shaped filter entry for native open/save dialogs. Pairs a
//! human-readable category name ("VSIX Extensions") with a list of
//! extensions (`["vsix"]`). Consumed by `tauri_plugin_dialog`'s
//! `add_filter(name, &[&str])` once the list is flattened.
//!
//! Kept as a bare data atom so both dialog variants (open/save) import
//! the same struct without pulling the full handler module.

#[derive(Debug, Clone)]
pub struct DialogFilter {
	pub Name:String,

	pub Extensions:Vec<String>,
}
