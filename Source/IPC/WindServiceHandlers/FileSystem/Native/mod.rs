
//! Native FS atoms - URI-aware, direct `tokio::fs`. Wind/Sky's `file:*`
//! channels route here.

pub mod FileCloneNative;

pub mod FileCloseFd;

pub mod FileOpenFd;

pub mod FileDeleteNative;

pub mod FileExistsNative;

pub mod FileMkdirNative;

pub mod FileReaddirNative;

pub mod FileReadNative;

pub mod FileRealpath;

pub mod FileRenameNative;

pub mod FileStatNative;

pub mod FileUnwatch;

pub mod FileWatch;

pub mod FileWriteNative;
