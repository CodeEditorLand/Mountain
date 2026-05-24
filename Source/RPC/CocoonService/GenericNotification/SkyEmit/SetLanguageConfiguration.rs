use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::Struct;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let _ = Env.ApplicationHandle.emit("sky://language/configure", &Params);
}
