//! URL serialization helpers for JSON persistence. Handles encoding and
//! decoding of URI strings across state boundaries.

/// URL encoder (struct fields to query-parameter format).
pub mod URLSerializer;

/// URL decoder (query-parameter format back to struct fields).
pub mod URLDeserializer;
