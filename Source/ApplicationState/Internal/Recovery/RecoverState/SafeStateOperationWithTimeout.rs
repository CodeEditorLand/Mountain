//! Run a synchronous, blocking state operation off-thread with a hard
//! timeout. The thread is allowed to finish in the background after
//! the timeout fires; only the receiver gives up. Used during
//! recovery where a hung repair must not stall the main runtime.

use CommonLibrary::Error::CommonError::CommonError;

use crate::dev_log;

/// fn.
pub fn Fn<T, F>(Operation:F, TimeoutMs:u64, OperationName:&str) -> Result<T, CommonError>
where
	F: FnOnce() -> Result<T, CommonError> + Send + 'static,
	T: Send + 'static, {
	let (Sender, Receiver) = std::sync::mpsc::channel();

	std::thread::spawn(move || {
		let _ = Sender.send(Operation());
	});

	match Receiver.recv_timeout(std::time::Duration::from_millis(TimeoutMs)) {
		Ok(Result) => Result,

		Err(_) => {
			dev_log!(
				"lifecycle",
				"error: [RecoverState] Operation '{}' timed out after {}ms",
				OperationName,
				TimeoutMs
			);

			Err(CommonError::Unknown { Description:format!("Operation '{}' timed out", OperationName) })
		},
	}
}
