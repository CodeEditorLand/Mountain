//! Filter a state map in-place by a validator predicate. Logs at warn
//! level when entries are removed so corruption is visible without
//! drowning the recovery path in chatter when nothing changes.

use std::collections::HashMap;

use crate::dev_log;

/// fn.
pub fn Fn<T>(StateData:&mut HashMap<String, T>, Validator:impl Fn(&T) -> bool) {
	let OriginalLen = StateData.len();

	StateData.retain(|_, Value| Validator(Value));

	let RemovedCount = OriginalLen - StateData.len();

	if RemovedCount > 0 {
		dev_log!(
			"lifecycle",
			"warn: [RecoverState] Removed {} invalid state entries ({} remaining)",
			RemovedCount,
			StateData.len()
		);
	}
}
