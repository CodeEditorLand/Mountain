use serde_json::Value;
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let _ = Env.ApplicationHandle.emit_to("main", "sky://language/configure", &Params);
}
