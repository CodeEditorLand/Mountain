//! Categories of feature flags. Determines flag scope/audience and is used
//! to bulk-query the registry by group.

#[derive(Debug, Clone, PartialEq)]
/// Enumeration for the enclosing variant set.
pub enum Enum {
	/// Experimental features (may change or be removed)
	Experimental,

	/// Legacy features (can be disabled)
	Legacy,

	/// Performance-sensitive features
	Performance,

	/// User-facing features
	UserFacing,

	/// Internal/developer features
	Internal,
}
