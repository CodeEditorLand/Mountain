//! # TreeView (Tauri command surface)
//!
//! Bridges tree-view UI requests from Sky (file explorer, SCM
//! viewlet, debug viewlet, extension-contributed views) into the
//! `MountainEnvironment::Require<dyn TreeViewProvider>` registry.
//! Eight wire-bound commands, each in its own file (file name =
//! Tauri command identifier per the Naming-Convention exception):
//!
//! - `GetTreeViewChildren::Fn` - fetch children for a tree
//!   node (or root).
//! - `GetTreeViewItem::Fn` - fetch a single item's metadata.
//! - `OnTreeViewExpansionChanged::Fn` (stub).
//! - `OnTreeViewSelectionChanged::Fn` (stub).
//! - `RefreshTreeView::Fn` - request data refresh.
//! - `RevealTreeViewItem::Fn` - focus / scroll-into -view.
//! - `PersistTreeView::Fn` (stub).
//! - `RestoreTreeView::Fn` (stub).
//!
//! Errors propagate as `Result<Value, String>` with the error
//! string surfaced directly to the renderer.
//!
//! VS Code reference:
//! `vs/workbench/api/browser/mainThreadTreeViews.ts`,
//! `vs/workbench/api/common/extHostTreeViews.ts`.
//!
//! ## Planned Work
//!
//! - Trait additions on `CommonTreeViewProvider` for the four stubs (expansion,
//!   selection, persist, restore)
//! - Drag-and-drop, multi-column, badge / tooltip / icon-theming support
//! - Tree-item validation

pub mod GetTreeViewChildren;

pub mod GetTreeViewItem;

pub mod OnTreeViewExpansionChanged;

pub mod OnTreeViewSelectionChanged;

pub mod PersistTreeView;

pub mod RefreshTreeView;

pub mod RestoreTreeView;

pub mod RevealTreeViewItem;
