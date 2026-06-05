//! Persist the global memento to disk before the runtime tears down. Creates
//! the parent directory if missing.

use CommonLibrary::Error::CommonError::CommonError;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

impl ApplicationRunTime {
	pub async fn SaveApplicationState(&self) -> Result<(), CommonError> {
		dev_log!("lifecycle", "[ApplicationRunTime] Saving application state...");

		let GlobalMementoGuard = self
			.Environment
			.ApplicationState
			.Configuration
			.MementoGlobalStorage
			.lock();

		let GlobalMementoPath = self
			.Environment
			.ApplicationState
			.GlobalMementoPath
			.lock()
			.clone();

		if let Some(Parent) = GlobalMementoPath.parent() {
			if !Parent.exists() {
				std::fs::create_dir_all(Parent)
					.map_err(|E| CommonError::FileSystemIO { Path:Parent.to_path_buf(), Description:E.to_string() })?;
			}
		}

		let MementoJSON = serde_json::to_string_pretty(&*GlobalMementoGuard)
			.map_err(|E| CommonError::SerializationError { Description:E.to_string() })?;

		std::fs::write(&GlobalMementoPath, MementoJSON)
			.map_err(|E| CommonError::FileSystemIO { Path:GlobalMementoPath.clone(), Description:E.to_string() })
	}
}