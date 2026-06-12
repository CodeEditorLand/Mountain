//! File-system-domain handlers for `CocoonService`. Eleven entry points
//! covering read/write/stat, directory ops, watch, glob/text search, and
//! delete/rename/copy/create-directory.
/// CopyFile handler: copies a file from source to destination.
pub mod CopyFile;

/// CreateDirectory handler: creates a new directory.
pub mod CreateDirectory;

/// DeleteFile handler: deletes a file.
pub mod DeleteFile;

/// FindFiles handler: searches for files matching a glob pattern.
pub mod FindFiles;

/// FindTextInFiles handler: searches for text content across files.
pub mod FindTextInFiles;

/// ReadFile handler: reads the contents of a file.
pub mod ReadFile;

/// Readdir handler: lists entries in a directory.
pub mod Readdir;

/// RenameFile handler: renames a file or directory.
pub mod RenameFile;

/// Stat handler: retrieves file metadata.
pub mod Stat;

/// WatchFile handler: watches a file for changes.
pub mod WatchFile;

/// WriteFile handler: writes data to a file.
pub mod WriteFile;
