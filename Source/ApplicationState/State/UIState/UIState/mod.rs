pub mod GetPendingRequests;
pub mod AddPendingRequest;
pub mod RemovePendingRequest;
pub mod ClearAll;
pub mod Count;
pub mod Contains;

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use CommonLibrary::Error::CommonError::CommonError;
use crate::dev_log;

/// User interface request state containing pending UI interactions.
#[derive(Clone)]
pub struct Struct {
	/// Pending user interface request organized by request ID.
	///
	/// Each request has a oneshot sender for sending the response back.
	pub PendingUserInterfaceRequest:
		Arc<StandardMutex<HashMap<String, tokio::sync::oneshot::Sender<Result<serde_json::Value, CommonError>>>>>,
}
