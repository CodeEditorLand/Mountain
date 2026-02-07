//! # CocoonServiceServer
//! 
//! Sets up the gRPC server and injects the dependencies (Spine implementations).

use std::sync::Arc;
use tonic::transport::Server;
use crate::Vine::Generated::Vine::mountain_service_server::MountainServiceServer;
use crate::Vine::Server::CocoonServiceImpl::CocoonServiceImpl;
use crate::Core::Impl::DesktopSpine::{DesktopFileSystem, TauriWindowManager, DesktopLifecycle};
use crate::Core::Spine::{FileSystemSpine, WindowManagerSpine, LifecycleSpine};

pub async fn StartCocoonServiceServer(app_handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    
    // 1. Create Spine Implementations
    let fs_spine = Arc::new(DesktopFileSystem);
    let window_spine = Arc::new(TauriWindowManager { app_handle });
    let lifecycle_spine = Arc::new(DesktopLifecycle);

    // 2. Inject into Service
    let cocoon_service = CocoonServiceImpl::new(fs_spine, window_spine, lifecycle_spine);

    println!("[CocoonServiceServer] Listening on {}", addr);

    // 3. Start Server
    Server::builder()
        .add_service(MountainServiceServer::new(cocoon_service))
        .serve(addr)
        .await?;

    Ok(())
}
