

/**
 * @module error (Vine)
 * @description Aggregates and re-exports all error types specific to the
 * Vine gRPC communication layer.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod VineError;
pub use self::VineError::VineError;
