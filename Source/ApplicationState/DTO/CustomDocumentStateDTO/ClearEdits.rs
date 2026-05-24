//! `CustomDocumentStateDTO::ClearEdits`

use super::Struct;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::Utility::Serialization::URLSerializationHelper;

pub fn Fn(This:&mut Struct) { This.Edits.clear(); }
