#![allow(non_snake_case)]

//! Verify every gate listed in `RequiredGates` is enabled. Returns a
//! diagnostic string naming the missing gates so callers can surface
//! them in error UI.

use crate::Telemetry::Gates::GetRuntimeGates;

pub fn Fn(FeatureName:&str, RequiredGates:&[&str]) -> Result<(), String> {
	let Enabled = GetRuntimeGates::Fn();

	let Missing:Vec<_> = RequiredGates.iter().filter(|Gate| !Enabled.contains(**Gate)).collect();

	if Missing.is_empty() {
		Ok(())
	} else {
		Err(format!("Feature '{}' requires gates: {:?}", FeatureName, Missing))
	}
}
