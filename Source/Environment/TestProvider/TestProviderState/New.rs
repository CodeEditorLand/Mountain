//! `TestProviderState::New`

use super::Struct;
use std::collections::HashMap;
use crate::Environment::TestProvider::{TestControllerState, TestRun};

pub fn Fn() -> Struct { Self { Controllers:HashMap::new(), ActiveRuns:HashMap::new() } }
