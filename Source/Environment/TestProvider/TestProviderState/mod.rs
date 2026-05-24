pub mod New;

use std::collections::HashMap;
use crate::Environment::TestProvider::{TestControllerState, TestRun};

#[derive(Debug)]
pub struct Struct {
	pub Controllers:HashMap<String, TestControllerState::Struct>,

	pub ActiveRuns:HashMap<String, TestRun::Struct>,
}
