pub mod DisposeTerminalsSafely;

use std::sync::Arc;
use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	Terminal::TerminalProvider::TerminalProvider as TerminalProviderTrait,
};
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

#[derive(Debug, Clone)]
pub struct Struct;
