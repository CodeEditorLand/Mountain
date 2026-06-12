//! `GetMetrics` response DTO.

use crate::Air::AirServiceTypesStub::AirMetricsProtoDTO;

#[derive(Debug, Clone)]
/// Data for struct.
pub struct Struct {
	pub metrics:AirMetricsProtoDTO::Struct,

	pub error:String,
}
