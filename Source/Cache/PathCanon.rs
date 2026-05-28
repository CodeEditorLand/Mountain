//! Mountain-side compat surface for [`::Cache::PathCanon`]. Only the
//! [`Canonicalize`] entry-point still has direct Mountain callers
//! (`Environment::Utility::PathSecurity`); reach for the canonical
//! `::Cache::PathCanon::*` paths elsewhere.

pub mod Canonicalize;
