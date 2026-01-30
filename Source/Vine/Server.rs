//! # Vine Server
//!
//! Implements the gRPC server for Mountain-Cocoon communication.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;
use log::{debug, error, info};
use tonic::{Request, Response, Status};
use async_trait::async_trait;

use super::Generated::{
    cocoon_service_server::CocoonService,
    GenericRequest, GenericResponse, GenericNotification, CancelOperationRequest, Empty, RpcError
};
use crate::{
    ApplicationState::ApplicationState,
    Environment::MountainEnvironment,
    ExtensionManagement::ExtensionManagementService,
};

/// Implementation of the CocoonService gRPC server
pub struct CocoonServiceServer {
    /// Mountain environment
    environment: Arc<MountainEnvironment>,
}

impl CocoonServiceServer {
    /// Creates a new instance of the CocoonService server
    pub fn new(environment: Arc<MountainEnvironment>) -> Self {
        info!("[CocoonServiceServer] New instance created");
        
        Self {
            environment,
        }
    }
    
    /// Handle generic Mountain requests from Cocoon
    async fn handle_mountain_request(
        &self,
        request: GenericRequest,
    ) -> Result<GenericResponse, Status> {
        debug!(
            "[CocoonServiceServer] Handling Mountain request '{}' with ID {}",
            request.method, request.request_identifier
        );
        
        match request.method.as_str() {
            "InitializeExtensionHost" => {
                info!("[CocoonServiceServer] Initializing extension host");
                
                // Return success response
                Ok(GenericResponse {
                    request_identifier: request.request_identifier,
                    result: serde_json::to_vec(&"initialized").map_err(|e| {
                        Status::internal(format!("Failed to serialize response: {}", e))
                    })?,
                    error: None,
                })
            }
            
            "GetExtensions" => {
                debug!("[CocoonServiceServer] Getting extensions");
                
                let extension_service: Arc<dyn ExtensionManagementService> = self.environment.Require();
                let extensions = extension_service.get_extensions().await.map_err(|e| {
                    Status::internal(format!("Failed to get extensions: {}", e))
                })?;
                
                Ok(GenericResponse {
                    request_identifier: request.request_identifier,
                    result: serde_json::to_vec(&extensions).map_err(|e| {
                        Status::internal(format!("Failed to serialize extensions: {}", e))
                    })?,
                    error: None,
                })
            }
            
            "ActivateExtension" => {
                debug!("[CocoonServiceServer] Activating extension");
                
                let params: serde_json::Value = serde_json::from_slice(&request.parameter)
                    .map_err(|e| Status::invalid_argument(format!("Invalid parameters: {}", e)))?;
                
                let extension_id = params["extensionId"]
                    .as_str()
                    .ok_or_else(|| Status::invalid_argument("Missing extensionId parameter"))?;
                
                let extension_service: Arc<dyn ExtensionManagementService> = self.environment.Require();
                extension_service.activate_extension(extension_id).await
                    .map_err(|e| Status::internal(format!("Failed to activate extension: {}", e)))?;
                
                Ok(GenericResponse {
                    request_identifier: request.request_identifier,
                    result: serde_json::to_vec(&"activated").map_err(|e| {
                        Status::internal(format!("Failed to serialize response: {}", e))
                    })?,
                    error: None,
                })
            }
            
            _ => {
                error!(
                    "[CocoonServiceServer] Unknown Mountain request method '{}'",
                    request.method
                );
                
                Err(Status::unimplemented(format!(
                    "Unknown method: {}",
                    request.method
                )))
            }
        }
    }
}

#[async_trait]
impl CocoonService for CocoonServiceServer {
    /// Process Mountain requests from Cocoon
    async fn process_mountain_request(
        &self,
        request: Request<GenericRequest>,
    ) -> Result<Response<GenericResponse>, Status> {
        let request_data = request.into_inner();
        
        match self.handle_mountain_request(request_data).await {
            Ok(response) => Ok(Response::new(response)),
            Err(status) => {
                error!("[CocoonServiceServer] Error processing Mountain request: {}", status);
                Err(status)
            }
        }
    }
    
    /// Send Mountain notifications to Cocoon
    async fn send_mountain_notification(
        &self,
        request: Request<GenericNotification>,
    ) -> Result<Response<Empty>, Status> {
        let notification = request.into_inner();
        
        debug!(
            "[CocoonServiceServer] Received Mountain notification '{}'",
            notification.method
        );
        
        // Handle notifications (fire-and-forget)
        match notification.method.as_str() {
            "ExtensionActivated" => {
                info!("[CocoonServiceServer] Extension activated notification");
            }
            
            "ExtensionDeactivated" => {
                info!("[CocoonServiceServer] Extension deactivated notification");
            }
            
            _ => {
                debug!(
                    "[CocoonServiceServer] Unknown Mountain notification '{}'",
                    notification.method
                );
            }
        }
        
        Ok(Response::new(Empty {}))
    }
    
    /// Cancel operations requested by Mountain
    async fn cancel_operation(
        &self,
        request: Request<CancelOperationRequest>,
    ) -> Result<Response<Empty>, Status> {
        let cancel_request = request.into_inner();
        
        info!(
            "[CocoonServiceServer] Cancelling operation with ID {}",
            cancel_request.request_identifier_to_cancel
        );
        
        // TODO: Implement operation cancellation logic
        
        Ok(Response::new(Empty {}))
    }
}
