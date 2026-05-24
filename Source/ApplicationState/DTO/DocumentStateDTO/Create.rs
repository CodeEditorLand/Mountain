//! `DocumentStateDTO::Create`

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

pub fn Fn(URI:Url, LanguageIdentifier:Option<String>, Content:String) -> Result<Self, CommonError> {
		// Validate URI is not empty
		if URI.as_str().is_empty() {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"URI".into(),
				Reason:"URI cannot be empty".into(),
			});
		}

		let LanguageID = LanguageIdentifier.unwrap_or_else(|| "plaintext".to_string());

		// Validate language identifier length
		if LanguageID.len() > MAX_LANGUAGE_ID_LENGTH {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"LanguageIdentifier".into(),
				Reason:format!("Language identifier exceeds maximum length of {} bytes", MAX_LANGUAGE_ID_LENGTH),
			});
		}

		let (Lines, EOL) = AnalyzeTextLinesAndEOL(&Content);

		// Validate document line count
		if Lines.len() > MAX_DOCUMENT_LINES {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Content".into(),
				Reason:format!("Document exceeds maximum line count of {}", MAX_DOCUMENT_LINES),
			});
		}

		// Validate individual line lengths
		for (Index, Line) in Lines.iter().enumerate() {
			if Line.len() > MAX_LINE_LENGTH {
				return Err(CommonError::InvalidArgument {
					ArgumentName:"Content".into(),
					Reason:format!("Line {} exceeds maximum length of {} bytes", Index + 1, MAX_LINE_LENGTH),
				});
			}
		}

		let Encoding = "utf8".to_string();

		Ok(Self {
			URI,

			LanguageIdentifier:LanguageID,

			Version:1,

			Lines,

			EOL,

			IsDirty:false,

			Encoding,

			VersionIdentifier:1,
		})
	}
