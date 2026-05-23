//! Per-test outcome: identifier, fully-qualified name, status, optional
//! duration in ms, and optional error/stack-trace pair populated on
//! failures.

use serde::{Deserialize, Serialize};

use crate::Environment::TestProvider::TestRunStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub TestIdentifier:String,

	pub FullName:String,

	pub Status:TestRunStatus::Enum,

	pub DurationMs:Option<u64>,

	pub ErrorMessage:Option<String>,

	pub StackTrace:Option<String>,
}
