

/**
 * @module indexing (Handlers)
 * @description This module contains the logic and specialized runtime for background
 * file indexing tasks. It demonstrates how to create a limited-capability
 * environment for improved security and robustness.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod IndexerEnvironment;
mod IndexerLogic;
mod IndexerRuntime;

pub use self::IndexerLogic::StartIndexingEffect; // Expose a top-level effect
