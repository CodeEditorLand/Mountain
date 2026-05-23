
//! Removes a completed PID from the registry.

pub fn Fn(OperationId:&str) {
	if OperationId.is_empty() {
		return;
	}

	if let Ok(mut Map) = super::running_processes().lock() {
		Map.remove(OperationId);
	}
}
