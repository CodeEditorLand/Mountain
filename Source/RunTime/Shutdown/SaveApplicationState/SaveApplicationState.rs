//! `SaveApplicationState::SaveApplicationState`

use super::Struct;
use CommonLibrary::Error::CommonError::CommonError;
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub fn Fn(This:&Struct) -> Result<(), CommonError> {
		dev_log!("lifecycle", "[ApplicationRunTime] Saving application state...");

		let GlobalMementoGuard = self
			.Environment
			.ApplicationState
			.Configuration
			.MementoGlobalStorage
			.lock()
			.map_err(|E| CommonError::StateLockPoisoned { Context:E.to_string() })?;

		let GlobalMementoPath = self
			.Environment
			.ApplicationState
			.GlobalMementoPath
			.lock()
			.map_err(|E| CommonError::StateLockPoisoned { Context:E.to_string() })?
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
