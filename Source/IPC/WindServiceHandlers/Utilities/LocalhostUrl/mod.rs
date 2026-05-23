//! The localhost plugin base URL (`http://localhost:<port>`).
//! State held here; `Get` and `Set` expose atomic accessors.

pub(crate) static URL:std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub mod Get;

pub mod Set;
