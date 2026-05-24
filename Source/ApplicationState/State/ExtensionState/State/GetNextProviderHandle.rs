//! `Struct::GetNextProviderHandle`

use super::Struct;
use std::sync::Arc;
use tokio::sync::Notify;
use super::{ExtensionRegistry, ProviderRegistration, ScannedExtensions};
use crate::dev_log;

pub fn Fn(This:&Struct) -> u32 { This.Registry.GetNextProviderHandle() }
