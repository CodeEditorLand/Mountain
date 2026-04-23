#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Managed FS atoms - route via Application runtime's
//! `FileSystemReader`/`FileSystemWriter` trait objects.

pub mod FileCopy;
pub mod FileDelete;
pub mod FileExists;
pub mod FileMkdir;
pub mod FileMove;
pub mod FileRead;
pub mod FileReadBinary;
pub mod FileReaddir;
pub mod FileStat;
pub mod FileWrite;
pub mod FileWriteBinary;
