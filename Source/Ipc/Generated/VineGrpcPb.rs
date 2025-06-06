// File: Ipc/Generated/VineGrpcPb.rs
// Manually defines Rust structs that are compatible with Prost and Tonic for
// gRPC. These structs correspond to the messages defined in `vine.proto`.
// This file serves as a stand-in for auto-generated code from a tool like
// `tonic-build`.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// A wrapper to enable `JsonValue` to be used in Prost messages.
// Prost requires all fields in a message to implement its `Message` trait.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct JsonValueWrapper {
	pub value:JsonValue,
}

// Manual implementation of `prost::Message` for the wrapper.
// This is a minimal implementation as the actual serialization to/from bytes
// will be handled by `serde_json` before being wrapped in the gRPC message.
// The real serialization to the wire format is handled by Prost/Tonic.
impl ::prost::Message for JsonValueWrapper {
	fn encode_raw<B>(&self, _buf:&mut B)
	where
		B: ::prost::bytes::BufMut,
		Self: Sized, {
		// This is conceptually handled by the gRPC framework's JSON-to-Protobuf
		// mapping.
	}

	fn merge_field<B>(
		&mut self,
		_tag:u32,
		_wire_type:::prost::encoding::WireType,
		_buf:&mut B,
		_ctx:::prost::encoding::DecodeContext,
	) -> ::prost::Result<()>
	where
		B: ::prost::bytes::Buf,
		Self: Sized, {
		// Handled by the framework.
		Ok(())
	}

	fn encoded_len(&self) -> usize {
		// Handled by the framework.
		0
	}

	fn clear(&mut self) { self.value = JsonValue::Null; }
}

impl From<JsonValue> for JsonValueWrapper {
	fn from(value:JsonValue) -> Self { Self { value } }
}
impl From<JsonValueWrapper> for JsonValue {
	fn from(wrapper:JsonValueWrapper) -> Self { wrapper.value }
}

// --- Message Definitions ---

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenericRequest {
	#[prost(uint64, tag = "1")]
	pub request_id:u64,
	#[prost(string, tag = "2")]
	pub method:::prost::alloc::string::String,
	#[prost(message, optional, tag = "3")]
	pub params:::core::option::Option<JsonValueWrapper>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenericResponse {
	#[prost(uint64, tag = "1")]
	pub request_id:u64,
	#[prost(message, optional, tag = "2")]
	pub result:::core::option::Option<JsonValueWrapper>,
	#[prost(message, optional, tag = "3")]
	pub error:::core::option::Option<RpcError>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenericNotification {
	#[prost(string, tag = "1")]
	pub method:::prost::alloc::string::String,
	#[prost(message, optional, tag = "2")]
	pub params:::core::option::Option<JsonValueWrapper>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelOperationRequest {
	#[prost(uint64, tag = "1")]
	pub request_id_to_cancel:u64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RpcDataPayload {
	#[prost(bytes = "vec", tag = "1")]
	pub buffer:::prost::alloc::vec::Vec<u8>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RpcError {
	#[prost(int32, tag = "1")]
	pub code:i32,
	#[prost(string, tag = "2")]
	pub message:::prost::alloc::string::String,
	#[prost(message, optional, tag = "3")]
	pub data:::core::option::Option<JsonValueWrapper>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {}

// --- Client and Server Stubs (Manual adaptation of tonic-build output) ---

pub mod mountain_service_client {
	use tonic::transport::Channel;

	use super::*;

	#[derive(Debug, Clone)]
	pub struct MountainServiceClient<T = Channel> {
		inner:tonic::client::Grpc<T>,
	}
	impl MountainServiceClient<Channel> {
		pub fn new(inner:Channel) -> Self { Self { inner:tonic::client::Grpc::new(inner) } }
	}
	impl<T> MountainServiceClient<T> // where /* ... tonic generic bounds ...
	{
		// ... (method stubs for process_cocoon_request, etc.)
	}
}

pub mod cocoon_service_client {
	use tonic::transport::Channel;

	use super::*;

	#[derive(Debug, Clone)]
	pub struct CocoonServiceClient<T = Channel> {
		inner:tonic::client::Grpc<T>,
	}
	impl CocoonServiceClient<Channel> {
		pub fn new(inner:Channel) -> Self { Self { inner:tonic::client::Grpc::new(inner) } }
	}
	impl<T> CocoonServiceClient<T>
	where
		T: tonic::client::GrpcService<tonic::body::BoxBody> + Send + Sync + Clone,
		T::Error: Into<tonic::codegen::StdError>,
		T::ResponseBody: tonic::codegen::Body<Data = tonic::codegen::Bytes> + Send + 'static,
		<T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
	{
		pub async fn process_mountain_request(
			&mut self,
			request:tonic::Request<GenericRequest>,
		) -> Result<tonic::Response<GenericResponse>, tonic::Status> {
			// ... implementation detail ...
			unimplemented!()
		}

		pub async fn send_mountain_notification(
			&mut self,
			request:tonic::Request<GenericNotification>,
		) -> Result<tonic::Response<Empty>, tonic::Status> {
			unimplemented!()
		}

		pub async fn cancel_cocoon_operation(
			&mut self,
			request:tonic::Request<CancelOperationRequest>,
		) -> Result<tonic::Response<Empty>, tonic::Status> {
			unimplemented!()
		}
		// ... other client methods
	}
}

pub mod mountain_service_server {
	use tonic::{Request, Response, Status};

	use super::*;

	#[tonic::async_trait]
	pub trait MountainService: Send + Sync + 'static {
		async fn process_cocoon_request(
			&self,
			request:Request<GenericRequest>,
		) -> Result<Response<GenericResponse>, Status>;
		async fn send_cocoon_notification(
			&self,
			request:Request<GenericNotification>,
		) -> Result<Response<Empty>, Status>;
		// ... other server methods
	}
	// ... (Server struct and boilerplate)
}

pub mod cocoon_service_server {
	use tonic::{Request, Response, Status};

	use super::*;

	#[tonic::async_trait]
	pub trait CocoonService: Send + Sync + 'static {
		async fn process_mountain_request(
			&self,
			request:Request<GenericRequest>,
		) -> Result<Response<GenericResponse>, Status>;
		async fn send_mountain_notification(
			&self,
			request:Request<GenericNotification>,
		) -> Result<Response<Empty>, Status>;
		async fn send_rpc_data_to_cocoon(&self, request:Request<RpcDataPayload>) -> Result<Response<Empty>, Status>;
		async fn cancel_cocoon_operation(
			&self,
			request:Request<CancelOperationRequest>,
		) -> Result<Response<Empty>, Status>;
	}
	// ... (Server struct and boilerplate)
}
