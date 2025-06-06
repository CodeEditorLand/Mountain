
// This module defines and exports common Data Transfer Objects (DTOs) that
// might be shared across various RPC argument structures, such as glob
// patterns.

#![allow(non_snake_case, non_camel_case_types)]

mod GlobPattern;

pub use GlobPattern::GlobPattern;
