//! Returns `~/.fiddee/workspaces/RecentlyOpened.json`.

pub fn Fn() -> std::path::PathBuf {
	crate::IPC::WindServiceHandlers::Utilities::FiddeeRoot::Fn()
		.join("workspaces")
		.join("RecentlyOpened.json")
}
