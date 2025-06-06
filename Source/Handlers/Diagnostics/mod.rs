
// This module defines and exports handlers and data structures related to
// diagnostics management (e.g., problems, errors, warnings reported by linters
// or extensions).

mod Diagnostics; // Contains the main logic for handling diagnostic operations
pub mod DiagnosticsEntry; // DTO for a single diagnostic entry (URI + markers)
pub mod MarkerData; // DTO for individual diagnostic markers
pub mod UriComponentsFilter; // DTO for filtering diagnostics by URI components

pub use self::Diagnostics::*; // Re-export all public items from Diagnostics.rs
// Individual DTOs are typically not re-exported at this level unless they are
// fundamental types used widely outside this specific handler context.
// Callers would use Handlers::Diagnostics::Dto::MarkerData etc. or directly
// import if needed.
