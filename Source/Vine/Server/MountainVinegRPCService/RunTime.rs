//! `MountainVinegRPCService::RunTime`

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

pub fn Fn(This:&Struct) -> &Arc<ApplicationRunTime> { &This.RunTime }
