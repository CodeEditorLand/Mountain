pub mod New;
pub mod IsInCategory;
pub mod Resource;
pub mod Action;

use serde::{Deserialize, Serialize};

/// Permission definition for RBAC
/// Permissions represent individual access rights that can be granted to roles.
/// They follow a naming convention of "resource.Action" (e.g., "file.read",
/// "config.update") for clear organization.
/// ## Permission Categories
/// - **file**: File system operations (read, write, delete)
/// - **config**: Configuration management (read, update)
/// - **storage**: Storage operations (read, write)
/// - **system**: System-level operations (external access)
/// ## Example Usage
/// ```rust,ignore
/// let permission = Struct {
///     name: "file.write".to_string(),
///     description: "Write file operations".to_string(),
///     category: "file".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	/// Unique permission identifier (e.g., "file.read", "config.update")
	pub name:String,

	/// Human-readable description of what this permission allows
	pub description:String,

	/// Category for groupings (e.g., "file", "config", "storage")
	pub category:String,
}
