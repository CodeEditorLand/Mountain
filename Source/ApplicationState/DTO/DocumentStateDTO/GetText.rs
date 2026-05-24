//! `DocumentStateDTO::GetText`

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

pub fn Fn(This:&Struct) -> String { This.Lines.join(&This.EOL) }
