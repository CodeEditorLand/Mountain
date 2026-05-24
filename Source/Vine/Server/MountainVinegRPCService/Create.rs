//! `MountainVinegRPCService::Create`

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

pub fn Fn(ApplicationHandle:AppHandle, RunTime:Arc<ApplicationRunTime>) -> Struct {
	dev_log!("grpc", "[MountainVinegRPCService] New instance created");

	Self {
		ApplicationHandle,

		RunTime,

		ActiveOperations:Arc::new(RwLock::new(HashMap::new())),
	}
}
