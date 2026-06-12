//! `GetMetrics` response DTO.

use crate::Air::AirServiceTypesStub::AirMetricsProtoDTO;

#[derive(Debug, Clone)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub metrics:AirMetricsProtoDTO::Struct,

	pub error:String,
}
