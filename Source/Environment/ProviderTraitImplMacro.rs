//! # ProviderTraitImplMacro (Environment)
//!
//! ## RESPONSIBILITIES
//! Provides a declarative macro to generate `Requires<T>` trait implementations for
//! MountainEnvironment, significantly reducing boilerplate code.
//!
//! ## ARCHITECTURAL ROLE
//! This macro is part of the Environment module's Dependency Injection (DI) system,
//! which enables components to request capabilities through traits. MountainEnvironment
//! implements `Requires<T>` for all 25+ provider traits, and this macro automates
//! the generation of those implementations.
//!
//! ## USAGE
//!
//! The macro is invoked at the module level to generate trait implementations:
//!
//! ```rust,ignore
//! impl_provider!(CommandExecutor);
//! impl_provider!(ConfigurationProvider);
//! impl_provider!(FileSystemReader);
//! ```
//!
//! This generates:
//!
//! ```rust,ignore
//! impl Requires<dyn CommandExecutor> for MountainEnvironment {
//!     fn Require(&self) -> Arc<dyn CommandExecutor> {
//!         Arc::new(self.clone())
//!     }
//! }
//!
//! impl Requires<dyn ConfigurationProvider> for MountainEnvironment {
//!     fn Require(&self) -> Arc<dyn ConfigurationProvider> {
//!         Arc::new(self.clone())
//!     }
//! }
//! ```
//!
//! ## KEY FEATURES
//!
//! - **Type Safety**: Generates type-safe implementations at compile time
//! - **Zero Overhead**: Generated code is identical to hand-written implementations
//! - **Maintainability**: Adding new providers requires only one macro invocation
//! - **Consistency**: Ensures all implementations follow the same pattern
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! The macro generates code with zero runtime overhead compared to hand-written
//! implementations. The macro expansion happens at compile time, and the generated
//! code is identical to what would be written manually.
//!
//! ## ERROR HANDLING
//!
//! The macro itself does not handle errors - any type checking or compilation errors
//! will be reported by the Rust compiler on the generated code.

/// Macro to generate `Requires<T>` trait implementations for MountainEnvironment.
///
/// This macro takes a trait name and generates an implementation of `Requires<dyn Trait>`
/// for `MountainEnvironment`. The implementation returns `Arc::new(self.clone())`,
/// which is the standard pattern for the DI container.
///
/// # Arguments
///
/// * `$trait_name` - The name of the trait (without `dyn` prefix)
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
///
/// # Note
///
/// This macro is intended to be used within `MountainEnvironment.rs` after the
/// `MountainEnvironment` struct definition. The macro assumes that `MountainEnvironment`
/// implements all the traits that are being requested.
#[macro_export]
macro_rules! impl_provider {
    ($trait_name:ident) => {
        impl Requires<dyn $trait_name> for MountainEnvironment {
            fn Require(&self) -> Arc<dyn $trait_name> {
                Arc::new(self.clone())
            }
        }
    };
}
