#![allow(non_snake_case)]

//! File-system-domain handlers for `CocoonService`. Eleven entry points
//! covering read/write/stat, directory ops, watch, glob/text search, and
//! delete/rename/copy/create-directory.

pub mod CopyFile;

pub mod CreateDirectory;

pub mod DeleteFile;

pub mod FindFiles;

pub mod FindTextInFiles;

pub mod ReadFile;

pub mod Readdir;

pub mod RenameFile;

pub mod Stat;

pub mod WatchFile;

pub mod WriteFile;
