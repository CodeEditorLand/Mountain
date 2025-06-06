
// Defines the ActionEffect struct, a fundamental unit for representing
// asynchronous operations that depend on an environment.

#![allow(non_snake_case, non_camel_case_types)]

use std::{future::Future, pin::Pin, sync::Arc};

use crate::Environment::Environment;

/// An ActionEffect encapsulates an asynchronous function that takes an
/// environment accessor and returns a Result. This pattern promotes a
/// functional, declarative style for
//  defining operations.
pub struct ActionEffect<AccessorType:?Sized, ErrorType, OutputType> {
	// The wrapped function. It's in an Arc to be cloneable without cloning the closure itself.
	Function:
		Arc<dyn Fn(AccessorType) -> Pin<Box<dyn Future<Output = Result<OutputType, ErrorType>> + Send>> + Send + Sync>,
}

impl<AccessorType:?Sized, ErrorType, OutputType> ActionEffect<AccessorType, ErrorType, OutputType> {
	/// Creates a new ActionEffect from a given function closure.
	pub fn New(
		Function:Arc<
			dyn Fn(AccessorType) -> Pin<Box<dyn Future<Output = Result<OutputType, ErrorType>> + Send>> + Send + Sync,
		>,
	) -> Self {
		Self { Function }
	}

	/// Applies the effect by executing its wrapped function with the provided
	/// accessor.
	pub async fn Apply(&self, Accessor:AccessorType) -> Result<OutputType, ErrorType>
	where
		AccessorType: Clone, {
		(self.Function)(Accessor).await
	}
}

impl<AccessorType:?Sized, ErrorType, OutputType> Clone for ActionEffect<AccessorType, ErrorType, OutputType> {
	/// Clones the ActionEffect, which only clones the Arc pointer to the
	/// function, not the function itself.
	fn clone(&self) -> Self { ActionEffect { Function:Arc::clone(&self.Function) } }
}
