// File: Common/HasEnvironment.rs
// Defines a generic trait for types that contain an environment.
// This is a conceptual file based on the `Haslanguagefeatureenvironment` trait,
// generalized to be applicable for any environment type.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use crate::Environment::{Environment, Requires}; // Assuming these traits are defined here

/// A generic trait for any type that holds an environment.
/// This is particularly useful for runtimes or other high-level contexts.
pub trait HasEnvironment {
	/// The specific type of the environment this struct holds.
	type EnvironmentType: Environment + Send + Sync;

	/// Gets a reference-counted pointer to the environment.
	fn GetEnvironment(&self) -> Arc<Self::EnvironmentType>;
}
