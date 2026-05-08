#![allow(non_snake_case)]

//! Workspace-domain handlers for `CocoonService`. Five entry points cover
//! document open/save, edit application, configuration changes, and
//! workspace-folder updates.

pub mod ApplyEdit;

pub mod OpenDocument;

pub mod SaveAll;

pub mod UpdateConfiguration;

pub mod UpdateWorkspaceFolders;
