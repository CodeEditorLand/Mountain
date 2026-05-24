//! `DocumentStateDTO::CreateUnsafe`

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

pub fn Fn(
		URI:Url,

		LanguageIdentifier:String,

		Lines:Vec<String>,

		EOL:String,

		IsDirty:bool,

		Encoding:String,

		Version:i64,

		VersionIdentifier:i64,
	) -> Struct {
		Self {
			URI,

			LanguageIdentifier,

			Version,

			Lines,

			EOL,

			IsDirty,

			Encoding,

			VersionIdentifier,
		}
	}
