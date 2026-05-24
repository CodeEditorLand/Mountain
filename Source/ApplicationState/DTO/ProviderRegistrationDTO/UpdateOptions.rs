//! `ProviderRegistrationDTO::UpdateOptions`

use super::Struct;
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn Fn(This:&mut Struct, Options:Value) { This.Options = Some(Options); }
