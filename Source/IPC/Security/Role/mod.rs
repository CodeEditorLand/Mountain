pub mod New;
pub mod HasPermission;
pub mod AddPermission;
pub mod RemovePermission;
pub mod PermissionCount;

use serde::{Deserialize, Serialize};

/// Role definition for RBAC
/// Roles are collections of permissions that can be assigned to users.
/// They provide a convenient way to manage access control by grouping
/// related permissions together.
/// ## Role Hierarchy
/// ```text
/// admin (full access)
///     |
///     ├── developer (read/write files, read config)
///     └── user (read-only access)
/// ```
/// ## Example Usage
/// ```rust,ignore
/// let role = Struct {
///     name: "editor".to_string(),
///     permissions: vec![
///         "file.read".to_string(),
///         "file.write".to_string(),
///         "storage.read".to_string(),
///     ],
///     description: "Editor role with file access".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	/// Unique role identifier
	pub name:String,

	/// List of permissions granted by this role
	pub permissions:Vec<String>,

	/// Human-readable description of the role
	pub description:String,
}
