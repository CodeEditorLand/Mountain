//! Atomically removes and returns a PID (for cancel operations).

pub fn Fn(OperationId:&str) -> Option<u32> {
	if OperationId.is_empty() {
		return None;
	}

	super::running_processes().lock().ok().and_then(|mut M| M.remove(OperationId))
}
