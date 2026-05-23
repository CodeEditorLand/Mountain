//! Registers a spawned PID under its OperationId.

pub fn Fn(OperationId:&str, Pid:u32) {
	if OperationId.is_empty() {
		return;
	}

	if let Ok(mut Map) = super::running_processes().lock() {
		Map.insert(OperationId.to_string(), Pid);
	}
}
