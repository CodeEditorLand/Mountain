//! # TreeView (Tauri command surface)
//!
//! Bridges tree-view UI requests from Sky (file explorer, SCM
//! viewlet, debug viewlet, extension-contributed views) into the
//! `MountainEnvironment::Require<dyn TreeViewProvider>` registry.
//! Eight wire-bound commands, each in its own file (file name =
//! Tauri command identifier per the Naming-Convention exception):
//!
//! - `GetTreeViewChildren::GetTreeViewChildren` - fetch children for a tree
//!   node (or root).
//! - `GetTreeViewItem::GetTreeViewItem` - fetch a single item's metadata.
//! - `OnTreeViewExpansionChanged::OnTreeViewExpansionChanged` (stub).
//! - `OnTreeViewSelectionChanged::OnTreeViewSelectionChanged` (stub).
//! - `RefreshTreeView::RefreshTreeView` - request data refresh.
//! - `RevealTreeViewItem::RevealTreeViewItem` - focus / scroll-into -view.
//! - `PersistTreeView::PersistTreeView` (stub).
//! - `RestoreTreeView::RestoreTreeView` (stub).
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

/// Gettreeviewchildren module.
pub mod GetTreeViewChildren;

/// Gettreeviewitem module.
pub mod GetTreeViewItem;

/// Ontreeviewexpansionchanged module.
pub mod OnTreeViewExpansionChanged;

/// Ontreeviewselectionchanged module.
pub mod OnTreeViewSelectionChanged;

/// Persisttreeview module.
pub mod PersistTreeView;

/// Refreshtreeview module.
pub mod RefreshTreeView;

/// Restoretreeview module.
pub mod RestoreTreeView;

/// Revealtreeviewitem module.
pub mod RevealTreeViewItem;
