//! Native file explorer surface for the workspace sidebar. Mountain owns the
//! tree view provider; URIs flow through `CommonLibrary::FileSystem` traits.

/// File explorer TreeView provider for the workspace sidebar.
pub mod FileExplorerViewProvider;
