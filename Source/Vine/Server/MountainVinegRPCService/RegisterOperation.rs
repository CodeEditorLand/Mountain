//! `MountainVinegRPCService::RegisterOperation`

use std::{collections::HashMap, sync::Arc};

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use super::Struct;
use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track,
	Vine::Generated::{
		CancelOperationRequest,
		Empty,
		GenericNotification,
		GenericRequest,
		GenericResponse,
		RpcError as RPCError,
		mountain_service_server::MountainService,
	},
	dev_log,
};

pub fn Fn(This:&Struct, request_id:u64) -> tokio_util::sync::CancellationToken {
	let token = tokio_util::sync::CancellationToken::new();

	This.ActiveOperations.write().await.insert(request_id, token.clone());

	dev_log!(
		"grpc",
		"[MountainVinegRPCService] Registered operation {} for cancellation",
		request_id
	);

	token
}
