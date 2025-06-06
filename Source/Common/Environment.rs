// File: Common/Environment.rs
// Defines the core traits for the Dependency Injection (DI) system.
// - `Environment`: A marker trait for any environment context.
// - `Requires`: A trait that allows an environment to provide a specific
//   capability or service.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

/// A marker trait for any struct that represents an application environment.
/// All environment structs must be `Send`, `Sync`, and have a `'static`
/// lifetime.
pub trait Environment: Send + Sync + 'static {}

/// A trait that enables an environment (`Self`) to provide a specific
/// capability (`Capability`). This is the core of the DI mechanism, allowing
/// parts of the application to "require" a service without needing to know the
/// concrete implementation.
pub trait Requires<Capability:?Sized>: Environment {
	/// Returns the required capability, wrapped in an `Arc` for shared
	/// ownership.
	fn require(&self) -> Arc<Capability>;
}

/// Allows an `Arc<T>` where `T` is an `Environment` to also be treated as an
/// `Environment`.
impl<T:Environment + ?Sized> Environment for Arc<T> {}

/// Allows an `Arc<E>` to provide a capability if the inner environment `E` can
/// provide it. This enables chained requirements and easy passing of the
/// environment `Arc`.
impl<E:Requires<Capability> + ?Sized, Capability:?Sized> Requires<Capability> for Arc<E> {
	fn require(&self) -> Arc<Capability> {
		// Dereference the Arc to call the `require` method on the inner environment
		// `E`.
		(**self).require()
	}
}
