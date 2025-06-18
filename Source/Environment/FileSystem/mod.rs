// @module FileSystem (Environment)
// @description This module contains the concrete sub-Environment for filesystem
// operations, demonstrating a more structured approach where each capability
// domain has its own Environment struct and provider implementation.
// NOTE: This structure was superseded by the flatter provider model but is
// generated here for completeness from the original source file list. The
// primary implementation is now in
// `mountain/src/Environment/FileSystemProvider.rs`.

#![allow(non_snake_case)]

mod FileSystemEnvironment;
mod FileSystemProvider;

pub use self::FileSystemEnvironment::FileSystemEnvironment;
