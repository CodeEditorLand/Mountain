#![allow(non_snake_case)]

//! Save-intent hint. Actual disk write happens via
//! `TextfileWrite`; this command exists so Wind can clear the
//! editor's dirty-dot UI marker without writing twice. Logged
//! at the `vfs` tag so a save trace appears in `Mountain.dev.log`.

use std::sync::Arc;

use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn TextfileSave(_runtime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let Uri = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	dev_log!("vfs", "textFile:save uri={:?}", Uri);

	Ok(Value::Null)
}
