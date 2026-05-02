#![allow(non_snake_case)]

//! `GetMetrics` response DTO.

use crate::Air::AirServiceTypesStub::AirMetricsProtoDTO;

#[derive(Debug, Clone)]
pub struct Struct {
	pub metrics:AirMetricsProtoDTO::Struct,
	pub error:String,
}
