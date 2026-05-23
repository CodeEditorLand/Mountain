//! Generates a UUID v4 string for use as a git operation ID.

pub fn Fn() -> String { uuid::Uuid::new_v4().to_string() }
