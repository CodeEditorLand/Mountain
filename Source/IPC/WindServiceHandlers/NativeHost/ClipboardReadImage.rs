#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `nativeHost:readImage`.
//! Binary image clipboard not yet implemented - returns empty array.

use serde_json::{Value, json};

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(json!([])) }
