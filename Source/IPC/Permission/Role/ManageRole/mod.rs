#![allow(non_snake_case)]

//! # ManageRole
//!
//! Role + permission types for the RBAC engine. Each `Role`
//! holds a deduplicated permission list; each `Permission`
//! lives in a `category.action` namespace and carries an
//! `IsSensitive` flag for elevated audit logging. The
//! `Create*` factories build the standard `user` /
//! `developer` / `admin` triple.

pub mod CreateAdminRole;

pub mod CreateDeveloperRole;

pub mod CreateStandardPermissions;

pub mod CreateStandardRoles;

pub mod CreateUserRole;

pub mod Permission;

pub mod Role;
