// File: Common/FileSystemDto.rs
// Defines Data Transfer Objects (DTOs) related to the filesystem,
// used for representing file types and metadata in a serializable format.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};

/// Represents the type of a filesystem entry.
/// This is a bit-flag enum, allowing an entry to be, for example, a file and a
/// symbolic link.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileType {
	Unknown = 0,
	File = 1,
	Directory = 2,
	SymbolicLink = 64,
}

/// Represents metadata about a file or directory.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FileSystemStat {
	// The type of the file (File, Directory, SymbolicLink).
	#[serde(rename = "Type")]
	pub FileType:u8, // Bit-flags from the FileType enum
	// Creation time in milliseconds since the UNIX epoch.
	#[serde(alias = "ctime")]
	pub CreationTime:u64,
	// Last modification time in milliseconds since the UNIX epoch.
	#[serde(alias = "mtime")]
	pub ModificationTime:u64,
	// The size of the file in bytes.
	pub Size:u64,
	// Optional. File permissions, typically represented as a Unix-style mode.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Permissions:Option<u32>,
}
