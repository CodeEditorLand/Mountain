// @module indexing (Handler)
// @description This module would contain the logic and specialized RunTime for
// background file indexing tasks. It demonstrates how to create a
// limited-capability Environment for improved security and robustness.
// NOTE: This feature is advanced and considered out of scope for the primary
// application logic, but the structure is preserved here.

#![allow(non_snake_case)]

mod IndexerEnvironment;
mod IndexerLogic;
mod IndexerRunTime;

// Expose a top-level effect to start the indexing process.
pub use self::IndexerLogic::StartIndexingEffect;
