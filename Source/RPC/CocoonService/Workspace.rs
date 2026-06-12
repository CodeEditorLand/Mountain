//! Workspace-domain handlers for `CocoonService`. Five entry points cover
//! document open/save, edit application, configuration changes, and
//! workspace-folder updates.
/// ApplyEdit handler: applies a text edit to an open document.
pub mod ApplyEdit;

/// OpenDocument handler: opens a document in the workspace.
pub mod OpenDocument;

/// SaveAll handler: saves all open documents.
pub mod SaveAll;

/// UpdateConfiguration handler: applies configuration changes from the
/// extension host.
pub mod UpdateConfiguration;

/// UpdateWorkspaceFolders handler: notifies the environment of workspace folder
/// changes.
pub mod UpdateWorkspaceFolders;
