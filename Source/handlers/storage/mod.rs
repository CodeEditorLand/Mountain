

/**
 * @module storage (Handlers)
 * @description This module contains the core logic for handling Memento-style
 * key-value storage operations, including reading from and writing to the
 * appropriate JSON storage files on disk.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod StorageLogic;

pub use self::StorageLogic::*;
