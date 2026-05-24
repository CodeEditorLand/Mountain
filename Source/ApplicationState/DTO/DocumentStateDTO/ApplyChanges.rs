//! `DocumentStateDTO::ApplyChanges`

use super::Struct;
use CommonLibrary::{Error::CommonError::CommonError, Utility::Serialization::URLSerializationHelper};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;
use crate::{
	ApplicationState::Internal::TextProcessing::AnalyzeTextLinesAndEOL::Fn as AnalyzeTextLinesAndEOL,
	dev_log,
};
use super::{RPCModelContentChangeDTO::RPCModelContentChangeDTO, RPCRangeDTO::RPCRangeDTO};

pub fn Fn(This:&mut Struct, NewVersion:i64, ChangesValue:&Value) -> Result<(), CommonError> {
		// Ignore stale changes.
		if NewVersion <= This.Version {
			return Ok(());
		}

		// Attempt to deserialize as an array of delta changes first.
		if let Ok(RPCChange) = serde_json::from_value::<Vec<RPCModelContentChangeDTO>>(ChangesValue.clone()) {
			dev_log!("model", "applying {} delta change(s) to document {}", RPCChange.len(), This.URI);

			This.Lines = ApplyDeltaChanges(&This.Lines, &This.EOL, &RPCChange);
		} else if let Some(FullText) = ChangesValue.as_str() {
			// If it's not deltas, check if it's a full text replacement.
			let (NewLines, NewEOL) = AnalyzeTextLinesAndEOL(FullText);

			This.Lines = NewLines;

			This.EOL = NewEOL;
		} else {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"ChangesValue".into(),

				Reason:format!(
					"Invalid change format for {}: expected string or RPCModelContentChangeDTO array.",
					This.URI
				),
			});
		}

		// Update metadata after changes have been applied.
		This.Version = NewVersion;

		This.VersionIdentifier += 1;

		This.IsDirty = true;

		Ok(())
	}
