//! `MergedConfigurationStateDTO::Create`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::dev_log;

pub fn Fn(Data:Value) -> Struct { Self { Data } }
