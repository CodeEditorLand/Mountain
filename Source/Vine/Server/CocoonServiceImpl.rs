//! # CocoonServiceImpl
//! 
//! The gRPC implementation for the Cocoon Extension Host.
//! This service acts as the "Limb" that connects the Cocoon process (Node.js)
//! to the Mountain Spine (Core traits) via the Vine Protocol.
//!
//! RESPONSIBILITIES:
//! 1. Receive `GenericRequest` from Cocoon (via gRPC).
//! 2. Decode the request parameters (JSON).
//! 3. Dispatch to the appropriate `Spine` implementation (Filesystem, Window, etc.).
//! 4. Encode the result back to `GenericResponse`.

use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::Vine::Generated::Vine::{
    mountain_service_server::MountainService, 
    GenericRequest, GenericResponse, GenericNotification
};

use crate::Core::Spine::{FileSystemSpine, WindowManagerSpine, LifecycleSpine, ClientInfo};

pub struct CocoonServiceImpl {
    // Injected dependencies (The Spine)
    pub fs_spine: Arc<dyn FileSystemSpine>,
    pub window_spine: Arc<dyn WindowManagerSpine>,
    pub lifecycle_spine: Arc<dyn LifecycleSpine>,
}

impl CocoonServiceImpl {
    pub fn new(
        fs_spine: Arc<dyn FileSystemSpine>,
        window_spine: Arc<dyn WindowManagerSpine>,
        lifecycle_spine: Arc<dyn LifecycleSpine>,
    ) -> Self {
        Self {
            fs_spine,
            window_spine,
            lifecycle_spine,
        }
    }
}

#[tonic::async_trait]
impl MountainService for CocoonServiceImpl {
    /// Process a Request/Response call from Cocoon
    async fn process_request(
        &self,
        request: Request<GenericRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let inner_req = request.into_inner();
        let method = inner_req.method.as_str();

        println!("[CocoonService] Received Request: {}", method);

        match method {
             // --- Lifecycle Spine (v0.3) ---
            "system.handshake" => {
                let info: ClientInfo = serde_json::from_slice(&inner_req.parameter)
                    .map_err(|e| Status::invalid_argument(format!("Invalid client info: {}", e)))?;
                
                let server_info = self.lifecycle_spine.handshake(info).await
                    .map_err(|e| Status::internal(e))?;
                    
                Ok(Response::new(GenericResponse {
                    request_id: inner_req.request_id,
                    payload: serde_json::to_vec(&server_info).unwrap(),
                    error: "".to_string(),
                    success: true,
                }))
            },

            // --- Filesystem Spine (v0.1) ---
            "fs.readFile" => {
                // Decode arguments: PathBuf
                let path: std::path::PathBuf = serde_json::from_slice(&inner_req.parameter)
                    .map_err(|e| Status::invalid_argument(format!("Invalid path: {}", e)))?;

                // Call Spine
                let result = self.fs_spine.read_file(path).await;

                match result {
                    Ok(content) => {
                        Ok(Response::new(GenericResponse {
                            request_id: inner_req.request_id,
                            payload: content, // Binary content
                            error: "".to_string(),
                            success: true,
                        }))
                    },
                    Err(e) => {
                         Ok(Response::new(GenericResponse {
                            request_id: inner_req.request_id,
                            payload: vec![],
                            error: e,
                            success: false,
                        }))
                    }
                }
            },

            // --- Window Manager Spine (v0.2) ---
            "window.showMessage" => {
                // Decode arguments: { title, message, level }
                #[derive(serde::Deserialize)]
                struct ShowMessageArgs {
                    title: String,
                    message: String,
                    level: String,
                }

                let args: ShowMessageArgs = serde_json::from_slice(&inner_req.parameter)
                    .map_err(|e| Status::invalid_argument(format!("Invalid args: {}", e)))?;

                // Call Spine (Fire and forget UI update)
                self.window_spine.show_message(&args.title, &args.message, &args.level).await;

                Ok(Response::new(GenericResponse {
                    request_id: inner_req.request_id,
                    payload: vec![],
                    error: "".to_string(),
                    success: true,
                }))
            },

            // --- Unimplemented ---
            _ => {
                eprintln!("[CocoonService] Unknown method: {}", method);
                Err(Status::not_found(format!("Method {} not found in Spine", method)))
            }
        }
    }

    /// Process a one-way Notification from Cocoon
    async fn process_notification(
        &self,
        request: Request<GenericNotification>,
    ) -> Result<Response<()>, Status> {
        let inner_notif = request.into_inner();
        println!("[CocoonService] Received Notification: {}", inner_notif.event);
        Ok(Response::new(()))
    }
}
