//! `WorkspaceFolderStateDTO::IsRoot`

use super::Struct;
use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::Utility::Serialization::URLSerializationHelper;

pub fn Fn(This:&Struct) -> bool { This.Index == 0 }
