#![allow(non_snake_case)]

//! Result of a Node binary resolution attempt. Carries both the path and the
//! source so log lines can distinguish shipped Node from system Node.

use std::path::PathBuf;

use crate::ProcessManagement::NodeResolver::NodeSource;

#[derive(Debug, Clone)]
pub struct Struct {
	pub Path:PathBuf,
	pub Source:NodeSource::Enum,
}
