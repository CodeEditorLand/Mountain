//! `DisposeTerminalsSafely::DisposeTerminalsSafely`

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	Terminal::TerminalProvider::TerminalProvider as TerminalProviderTrait,
};

use super::Struct;
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub fn Fn(This:&Struct) -> Result<(), CommonError> {
	let TerminalProvider:Arc<dyn TerminalProviderTrait> = This.Environment.Require();

	let TerminalIdentifiers:Vec<u64> = {
		let TerminalsGuard = self
			.Environment
			.ApplicationState
			.Feature
			.Terminals
			.ActiveTerminals
			.lock()
			.map_err(|E| CommonError::StateLockPoisoned { Context:E.to_string() })?;

		TerminalsGuard.keys().cloned().collect()
	};

	let mut DisposalErrors:Vec<String> = Vec::new();

	for Identifier in TerminalIdentifiers {
		match TerminalProvider.DisposeTerminal(Identifier).await {
			Ok(()) => {
				dev_log!(
					"lifecycle",
					"[ApplicationRunTime] Terminal {} disposed successfully",
					Identifier
				)
			},

			Err(Error) => {
				DisposalErrors.push(format!("Terminal {}: {}", Identifier, Error));

				dev_log!(
					"lifecycle",
					"warn: [ApplicationRunTime] Failed to dispose terminal {}: {}",
					Identifier,
					Error
				);
			},
		}
	}

	if !DisposalErrors.is_empty() {
		Err(CommonError::Unknown {
			Description:format!(
				"Terminal disposal completed with {} errors: {:?}",
				DisposalErrors.len(),
				DisposalErrors
			),
		})
	} else {
		Ok(())
	}
}
