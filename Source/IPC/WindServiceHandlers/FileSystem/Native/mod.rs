#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Native FS atoms - URI-aware, direct `tokio::fs`. Wind/Sky's `file:*`
//! channels route here.

pub mod FileCloneNative;

pub mod FileDeleteNative;

pub mod FileExistsNative;

pub mod FileMkdirNative;

pub mod FileReaddirNative;

pub mod FileReadNative;

pub mod FileRealpath;

pub mod FileRenameNative;

pub mod FileStatNative;

pub mod FileWriteNative;
