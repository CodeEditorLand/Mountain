pub mod New;
pub mod Get;
pub mod Set;

use std::sync::Arc;
use CommonLibrary::{Error::CommonError::CommonError, Storage::StorageProvider::StorageProvider};

pub struct Struct {
	pub(super) provider:Arc<dyn StorageProvider>,
}
