
//! Tree-view-domain handlers for `CocoonService`.
//! `RegisterTreeViewProvider::Fn`, `GetTreeChildren::Fn`, plus private
//! helpers `EnqueueTreeViewEmit` (16 ms emit batcher) and `ViewIdHandle`
//! (viewId → registration u32).

pub mod GetTreeChildren;

pub mod RegisterTreeViewProvider;

pub(crate) mod EnqueueTreeViewEmit;

pub(crate) mod ViewIdHandle;
