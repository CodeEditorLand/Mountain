//! Dispose every active PTY through `TerminalProvider::DisposeTerminal`.
//! Errors per terminal are collected; the loop never aborts early.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	Terminal::TerminalProvider::TerminalProvider as TerminalProviderTrait,
};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

impl ApplicationRunTime {
/// Disposes terminals safely.
	pub async fn DisposeTerminalsSafely(&self) -> Result<(), CommonError> {
		let TerminalProvider:Arc<dyn TerminalProviderTrait> = self.Environment.Require();

		let TerminalIdentifiers:Vec<u64> = {
			let TerminalsGuard = self.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock();

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
}
