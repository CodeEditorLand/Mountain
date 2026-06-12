//! Tree-view-domain handlers for `CocoonService`.
//! `GetTreeChildren::Fn`, `RegisterTreeViewProvider::Fn`, plus private
//! helpers `EnqueueTreeViewEmit` (16 ms emit batcher) and `ViewIdHandle`
//! (viewId → registration u32).
/// GetTreeChildren handler: retrieves child items for a tree view node.
pub mod GetTreeChildren;

/// RegisterTreeViewProvider handler: registers a tree view data provider.
pub mod RegisterTreeViewProvider;

pub(crate) mod EnqueueTreeViewEmit;

pub(crate) mod ViewIdHandle;
