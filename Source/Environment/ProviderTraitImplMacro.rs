//! # ProviderTraitImplMacro
//!
//! Declarative macro that generates `Requires<dyn T>` implementations for
//! `MountainEnvironment`, eliminating the boilerplate of writing an identical
//! `impl` block for each of the 25+ provider traits.
//!
//! Each invocation of `impl_provider!(TraitName)` expands to:
//!
//! ```rust,ignore
//! impl Requires<dyn TraitName> for MountainEnvironment {
//!     fn Require(&self) -> Arc<dyn TraitName> {
//!         Arc::new(self.clone())
//!     }
//! }
//! ```
//!
//! This is correct because `MountainEnvironment` directly implements every
//! provider trait, so cloning self and wrapping in `Arc` satisfies the
//! `Requires<dyn T>` contract. The generated code is identical to a
//! hand-written implementation - zero runtime overhead.
//!
//! Type safety and compilation errors for missing trait implementations are
//! reported by the Rust compiler on the generated `impl` block, not on the
//! macro call site.

/// Macro to generate `Requires<dyn T>` trait implementations for
/// `MountainEnvironment`.
///
/// # Arguments
///
/// * `$trait_name` - The name of the trait (without `dyn` prefix).
///
/// # Example
///
/// ```rust,ignore
/// impl_provider!(CommandExecutor);
/// ```
///
/// Expands to:
///
/// ```rust,ignore
/// impl Requires<dyn CommandExecutor> for MountainEnvironment {
///     fn Require(&self) -> Arc<dyn CommandExecutor> {
///         Arc::new(self.clone())
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_provider {

	($trait_name:ident) => {
		impl Requires<dyn $trait_name> for MountainEnvironment {
			fn Require(&self) -> Arc<dyn $trait_name> { Arc::new(self.clone()) }
		}
	};
}
