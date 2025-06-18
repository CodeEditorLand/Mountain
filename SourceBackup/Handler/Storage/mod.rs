// @module storage (Handler)
// @description This module contains the core logic for handling Memento-style
// key-value storage operations, including reading from and writing to the
// appropriate JSON storage files on disk.
//

#![allow(non_snake_case)]

mod StorageLogic;

pub use self::StorageLogic::*;
